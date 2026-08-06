#!/usr/bin/env python3
"""Code Janitor Runner — AST-based code smell scanner.

Scans Python files for common code hygiene issues:
- Unused imports
- Dead/unreachable code
- Oversized functions (> configurable line threshold)
- Too many parameters (> configurable threshold)
- Deep nesting (> configurable indentation depth)
- High cyclomatic complexity
- Missing docstrings
- Missing return type annotations
- Stale TODO/FIXME markers

Uses Python Standard Library only (ADR 0001).
"""

import argparse
import ast
import json
import re
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence

_REPO_ROOT = Path(__file__).resolve().parents[3] if len(Path(__file__).resolve().parents) > 3 else Path.cwd()
if str(_REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(_REPO_ROOT))

try:
    from scripts._path_safety import sanitize_path
except ImportError:

    def sanitize_path(p: Any, base_dir: Optional[Path] = None) -> Path:
        """Fallback path sanitizer when _path_safety is unavailable."""
        base = base_dir.resolve() if base_dir else Path.cwd().resolve()
        candidate = Path(p).resolve()
        try:
            candidate.relative_to(base)
        except ValueError as err:
            raise ValueError(f"Path '{p}' escapes allowed base directory '{base}'.") from err
        return candidate


try:
    from scripts._config_safety import load_skill_config
except ImportError:

    def load_skill_config(skill_name: str, cli_args: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """Fallback config loader when _config_safety is unavailable."""
        return cli_args or {}


# Default thresholds
DEFAULT_MAX_FUNCTION_LINES = 30
DEFAULT_MAX_PARAMETERS = 5
DEFAULT_MAX_NESTING_DEPTH = 4
DEFAULT_MAX_CYCLOMATIC_COMPLEXITY = 10

TODO_PATTERN = re.compile(r"#\s*(TODO|FIXME|HACK|XXX|NOQA)\b", re.IGNORECASE)


class SmellFinding:
    """Represents a single code smell finding."""

    def __init__(
        self,
        file_path: str,
        line: int,
        category: str,
        smell: str,
        severity: str,
        message: str,
        end_line: Optional[int] = None,
    ) -> None:
        self.file_path = file_path
        self.line = line
        self.end_line = end_line or line
        self.category = category
        self.smell = smell
        self.severity = severity
        self.message = message

    def to_dict(self) -> Dict[str, Any]:
        """Serialize finding to dictionary."""
        return {
            "file": self.file_path,
            "line": self.line,
            "end_line": self.end_line,
            "category": self.category,
            "smell": self.smell,
            "severity": self.severity,
            "message": self.message,
        }

    def __repr__(self) -> str:
        return f"SmellFinding({self.file_path}:L{self.line}, {self.severity} {self.smell}: {self.message})"


def _get_names_in_scope(tree: ast.AST) -> set:
    """Collect all Name nodes referenced in the AST (excluding import statements)."""
    names: set = set()
    for node in ast.walk(tree):
        if isinstance(node, ast.Name):
            names.add(node.id)
        elif isinstance(node, ast.Attribute):
            # Collect the root of attribute chains (e.g., os.path -> os)
            root = node
            while isinstance(root, ast.Attribute):
                root = root.value
            if isinstance(root, ast.Name):
                names.add(root.id)
    return names


def detect_unused_imports(tree: ast.AST, file_path: str) -> List[SmellFinding]:
    """Detect import statements whose names are never referenced."""
    findings: List[SmellFinding] = []
    used_names = _get_names_in_scope(tree)

    for node in ast.iter_child_nodes(tree):
        if isinstance(node, ast.Import):
            for alias in node.names:
                local_name = alias.asname if alias.asname else alias.name
                # For dotted imports like os.path, check the top-level name
                top_name = local_name.split(".")[0]
                if top_name not in used_names:
                    findings.append(
                        SmellFinding(
                            file_path=file_path,
                            line=node.lineno,
                            category="dispensables",
                            smell="unused-import",
                            severity="critical",
                            message=f"Unused import: '{alias.name}'.",
                        )
                    )
        elif isinstance(node, ast.ImportFrom):
            for alias in node.names:
                if alias.name == "*":
                    continue
                local_name = alias.asname if alias.asname else alias.name
                if local_name not in used_names:
                    module_name = node.module or ""
                    findings.append(
                        SmellFinding(
                            file_path=file_path,
                            line=node.lineno,
                            category="dispensables",
                            smell="unused-import",
                            severity="critical",
                            message=(f"Unused import: '{alias.name}' from '{module_name}'."),
                        )
                    )

    return findings


def _is_unconditional_exit(node: ast.AST) -> bool:
    """Check if an AST node is an unconditional exit statement."""
    return isinstance(node, (ast.Return, ast.Raise, ast.Break, ast.Continue))


def detect_dead_code(tree: ast.AST, file_path: str) -> List[SmellFinding]:
    """Detect unreachable code after return/raise/break/continue statements."""
    findings: List[SmellFinding] = []

    for node in ast.walk(tree):
        body: Optional[List[ast.stmt]] = None
        if isinstance(
            node,
            (ast.FunctionDef, ast.AsyncFunctionDef, ast.For, ast.While, ast.With),
        ):
            body = node.body
        elif isinstance(node, ast.If):
            body = node.body
        elif isinstance(node, ast.Try):
            body = node.body

        if body is None:
            continue

        for i, stmt in enumerate(body):
            if _is_unconditional_exit(stmt) and i < len(body) - 1:
                next_stmt = body[i + 1]
                findings.append(
                    SmellFinding(
                        file_path=file_path,
                        line=next_stmt.lineno,
                        end_line=getattr(next_stmt, "end_lineno", next_stmt.lineno),
                        category="dispensables",
                        smell="dead-code",
                        severity="critical",
                        message=(f"Unreachable code after '{type(stmt).__name__}' on line {stmt.lineno}."),
                    )
                )
                break

    return findings


def detect_oversized_functions(
    tree: ast.AST,
    file_path: str,
    max_lines: int = DEFAULT_MAX_FUNCTION_LINES,
) -> List[SmellFinding]:
    """Detect functions exceeding the maximum line count."""
    findings: List[SmellFinding] = []

    for node in ast.walk(tree):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            start = node.lineno
            end = getattr(node, "end_lineno", start)
            func_lines = end - start + 1
            if func_lines > max_lines:
                findings.append(
                    SmellFinding(
                        file_path=file_path,
                        line=start,
                        end_line=end,
                        category="bloaters",
                        smell="oversized-function",
                        severity="warning",
                        message=(f"Function '{node.name}' is {func_lines} lines (threshold: {max_lines})."),
                    )
                )

    return findings


def detect_too_many_parameters(
    tree: ast.AST,
    file_path: str,
    max_params: int = DEFAULT_MAX_PARAMETERS,
) -> List[SmellFinding]:
    """Detect functions with too many parameters."""
    findings: List[SmellFinding] = []

    for node in ast.walk(tree):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            args = node.args
            param_count = len(args.args) + len(args.posonlyargs) + len(args.kwonlyargs)
            # Exclude 'self' and 'cls' from count
            if args.args and args.args[0].arg in ("self", "cls"):
                param_count -= 1
            if param_count > max_params:
                findings.append(
                    SmellFinding(
                        file_path=file_path,
                        line=node.lineno,
                        category="bloaters",
                        smell="too-many-parameters",
                        severity="warning",
                        message=(f"Function '{node.name}' has {param_count} parameters (threshold: {max_params})."),
                    )
                )

    return findings


def _max_indent_depth(node: ast.AST, current_depth: int = 0) -> int:
    """Recursively calculate maximum nesting depth for control flow nodes."""
    max_depth = current_depth
    nesting_types = (ast.If, ast.For, ast.While, ast.With, ast.Try)

    for child in ast.iter_child_nodes(node):
        if isinstance(child, nesting_types):
            child_depth = _max_indent_depth(child, current_depth + 1)
            max_depth = max(max_depth, child_depth)
        else:
            child_depth = _max_indent_depth(child, current_depth)
            max_depth = max(max_depth, child_depth)

    return max_depth


def detect_deep_nesting(
    tree: ast.AST,
    file_path: str,
    max_depth: int = DEFAULT_MAX_NESTING_DEPTH,
) -> List[SmellFinding]:
    """Detect functions with control flow nesting exceeding the threshold."""
    findings: List[SmellFinding] = []

    for node in ast.walk(tree):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            depth = _max_indent_depth(node)
            if depth > max_depth:
                findings.append(
                    SmellFinding(
                        file_path=file_path,
                        line=node.lineno,
                        category="complexity",
                        smell="deep-nesting",
                        severity="warning",
                        message=(f"Function '{node.name}' has nesting depth {depth} (threshold: {max_depth})."),
                    )
                )

    return findings


def _count_branches(node: ast.AST) -> int:
    """Count branching statements within a function for cyclomatic complexity."""
    count = 0
    branch_types = (ast.If, ast.For, ast.While, ast.ExceptHandler)

    for child in ast.walk(node):
        if isinstance(child, branch_types):
            count += 1
        elif isinstance(child, ast.BoolOp):
            # Each 'and'/'or' adds a branch
            count += len(child.values) - 1

    return count


def detect_high_complexity(
    tree: ast.AST,
    file_path: str,
    max_complexity: int = DEFAULT_MAX_CYCLOMATIC_COMPLEXITY,
) -> List[SmellFinding]:
    """Detect functions with high cyclomatic complexity."""
    findings: List[SmellFinding] = []

    for node in ast.walk(tree):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            complexity = _count_branches(node) + 1  # +1 for the function entry
            if complexity > max_complexity:
                findings.append(
                    SmellFinding(
                        file_path=file_path,
                        line=node.lineno,
                        category="complexity",
                        smell="high-cyclomatic-complexity",
                        severity="warning",
                        message=(f"Function '{node.name}' has cyclomatic complexity {complexity} (threshold: {max_complexity})."),
                    )
                )

    return findings


def detect_missing_docstrings(tree: ast.AST, file_path: str) -> List[SmellFinding]:
    """Detect functions and classes without docstrings."""
    findings: List[SmellFinding] = []

    for node in ast.walk(tree):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            has_docstring = (
                node.body
                and isinstance(node.body[0], ast.Expr)
                and isinstance(node.body[0].value, ast.Constant)
                and isinstance(node.body[0].value.value, str)
            )
            if not has_docstring:
                kind = "Class" if isinstance(node, ast.ClassDef) else "Function"
                findings.append(
                    SmellFinding(
                        file_path=file_path,
                        line=node.lineno,
                        category="documentation",
                        smell="missing-docstring",
                        severity="advisory",
                        message=f"{kind} '{node.name}' has no docstring.",
                    )
                )

    return findings


def detect_missing_type_annotations(tree: ast.AST, file_path: str) -> List[SmellFinding]:
    """Detect function definitions without return type annotations."""
    findings: List[SmellFinding] = []

    for node in ast.walk(tree):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            if node.returns is None:
                findings.append(
                    SmellFinding(
                        file_path=file_path,
                        line=node.lineno,
                        category="documentation",
                        smell="missing-type-annotation",
                        severity="advisory",
                        message=(f"Function '{node.name}' has no return type annotation."),
                    )
                )

    return findings


def detect_stale_todos(source_lines: List[str], file_path: str) -> List[SmellFinding]:
    """Detect TODO, FIXME, HACK, XXX, and NOQA comments."""
    findings: List[SmellFinding] = []

    for i, line in enumerate(source_lines, start=1):
        match = TODO_PATTERN.search(line)
        if match:
            marker = match.group(1).upper()
            findings.append(
                SmellFinding(
                    file_path=file_path,
                    line=i,
                    category="maintenance",
                    smell="stale-todo",
                    severity="advisory",
                    message=f"Stale {marker} marker: {line.strip()}",
                )
            )

    return findings


def scan_file(
    file_path: Path,
    max_function_lines: int = DEFAULT_MAX_FUNCTION_LINES,
    max_parameters: int = DEFAULT_MAX_PARAMETERS,
    max_nesting_depth: int = DEFAULT_MAX_NESTING_DEPTH,
    max_cyclomatic_complexity: int = DEFAULT_MAX_CYCLOMATIC_COMPLEXITY,
) -> List[SmellFinding]:
    """Run all smell detectors on a single Python file."""
    content = file_path.read_text(encoding="utf-8")
    source_lines = content.splitlines()

    try:
        tree = ast.parse(content, filename=str(file_path))
    except SyntaxError as err:
        return [
            SmellFinding(
                file_path=str(file_path),
                line=err.lineno or 1,
                category="parse-error",
                smell="syntax-error",
                severity="critical",
                message=f"Failed to parse file: {err.msg}",
            )
        ]

    file_str = str(file_path)
    findings: List[SmellFinding] = []

    findings.extend(detect_unused_imports(tree, file_str))
    findings.extend(detect_dead_code(tree, file_str))
    findings.extend(detect_oversized_functions(tree, file_str, max_function_lines))
    findings.extend(detect_too_many_parameters(tree, file_str, max_parameters))
    findings.extend(detect_deep_nesting(tree, file_str, max_nesting_depth))
    findings.extend(detect_high_complexity(tree, file_str, max_cyclomatic_complexity))
    findings.extend(detect_missing_docstrings(tree, file_str))
    findings.extend(detect_missing_type_annotations(tree, file_str))
    findings.extend(detect_stale_todos(source_lines, file_str))

    findings.sort(key=lambda f: f.line)
    return findings


def scan_directory(
    dir_path: Path,
    max_function_lines: int = DEFAULT_MAX_FUNCTION_LINES,
    max_parameters: int = DEFAULT_MAX_PARAMETERS,
    max_nesting_depth: int = DEFAULT_MAX_NESTING_DEPTH,
    max_cyclomatic_complexity: int = DEFAULT_MAX_CYCLOMATIC_COMPLEXITY,
) -> List[SmellFinding]:
    """Run all smell detectors on all Python files in a directory (recursive)."""
    findings: List[SmellFinding] = []

    for py_file in sorted(dir_path.rglob("*.py")):
        findings.extend(
            scan_file(
                py_file,
                max_function_lines,
                max_parameters,
                max_nesting_depth,
                max_cyclomatic_complexity,
            )
        )

    return findings


def format_findings_text(findings: List[SmellFinding]) -> str:
    """Format findings as human-readable text."""
    if not findings:
        return "✅ No code smells detected. Code is clean!"

    severity_icons = {
        "critical": "🚨",
        "warning": "⚠️",
        "advisory": "💡",
    }

    lines: List[str] = []
    lines.append(f"Found {len(findings)} code smell(s):\n")

    for finding in findings:
        icon = severity_icons.get(finding.severity, "❓")
        lines.append(
            f"  {icon} {finding.severity.upper()} [{finding.smell}] {finding.file_path}:L{finding.line} — {finding.message}"
        )

    # Summary counts
    counts: Dict[str, int] = {}
    for f in findings:
        counts[f.severity] = counts.get(f.severity, 0) + 1

    lines.append("")
    lines.append("Summary:")
    for sev in ("critical", "warning", "advisory"):
        if sev in counts:
            icon = severity_icons.get(sev, "")
            lines.append(f"  {icon} {sev.upper()}: {counts[sev]}")

    return "\n".join(lines)


def parse_args(argv: Optional[Sequence[str]] = None) -> argparse.Namespace:
    """Parse command-line arguments."""
    parser = argparse.ArgumentParser(
        description="Code Janitor — AST-based code smell scanner",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    parser.add_argument(
        "--file",
        type=str,
        help="Path to a single Python file to scan.",
    )
    parser.add_argument(
        "--dir",
        type=str,
        help="Path to a directory to scan recursively.",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Output findings as JSON.",
    )
    parser.add_argument(
        "--max-function-lines",
        type=int,
        default=DEFAULT_MAX_FUNCTION_LINES,
        help="Maximum allowed lines per function.",
    )
    parser.add_argument(
        "--max-parameters",
        type=int,
        default=DEFAULT_MAX_PARAMETERS,
        help="Maximum allowed parameters per function.",
    )
    parser.add_argument(
        "--max-nesting-depth",
        type=int,
        default=DEFAULT_MAX_NESTING_DEPTH,
        help="Maximum allowed nesting depth.",
    )
    parser.add_argument(
        "--max-complexity",
        type=int,
        default=DEFAULT_MAX_CYCLOMATIC_COMPLEXITY,
        help="Maximum allowed cyclomatic complexity per function.",
    )
    return parser.parse_args(argv)


def main(argv: Optional[Sequence[str]] = None) -> int:
    """CLI entrypoint."""
    try:
        args = parse_args(argv)
    except SystemExit as exc:
        return exc.code if isinstance(exc.code, int) else 2

    if not args.file and not args.dir:
        sys.stderr.write("Error: Must specify --file or --dir.\n")
        return 1

    findings: List[SmellFinding] = []

    if args.file:
        try:
            safe_path = sanitize_path(args.file)
        except (ValueError, NameError):
            safe_path = Path(args.file).resolve()
        if not safe_path.is_file():
            sys.stderr.write(f"Error: File not found: {safe_path}\n")
            return 1
        findings.extend(
            scan_file(
                safe_path,
                args.max_function_lines,
                args.max_parameters,
                args.max_nesting_depth,
                args.max_complexity,
            )
        )

    if args.dir:
        try:
            safe_dir = sanitize_path(args.dir)
        except (ValueError, NameError):
            safe_dir = Path(args.dir).resolve()
        if not safe_dir.is_dir():
            sys.stderr.write(f"Error: Directory not found: {safe_dir}\n")
            return 1
        findings.extend(
            scan_directory(
                safe_dir,
                args.max_function_lines,
                args.max_parameters,
                args.max_nesting_depth,
                args.max_complexity,
            )
        )

    if args.json:
        output = {
            "total": len(findings),
            "findings": [f.to_dict() for f in findings],
        }
        print(json.dumps(output, indent=2))
    else:
        print(format_findings_text(findings))

    return 0


if __name__ == "__main__":
    sys.exit(main())
