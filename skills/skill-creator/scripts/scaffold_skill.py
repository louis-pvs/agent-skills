#!/usr/bin/env python3
"""Skill Scaffold and Validator Utility.

Automates the creation and validation of Agent Skills according to agentskills.io standard
and writing-great-skills design principles (Invocation modes, Router skills, Completion criteria).

Uses Python Standard Library only.
"""

import argparse
import ast
import os
import re
import sys
import unittest
from pathlib import Path
from typing import Dict, List, Tuple

STDLIB_MODULES = getattr(sys, "stdlib_module_names", None) or {
    "argparse",
    "ast",
    "asyncio",
    "base64",
    "collections",
    "concurrent",
    "configparser",
    "contextlib",
    "copy",
    "csv",
    "datetime",
    "decimal",
    "difflib",
    "doctest",
    "email",
    "enum",
    "functools",
    "glob",
    "hashlib",
    "hmac",
    "html",
    "http",
    "importlib",
    "inspect",
    "io",
    "json",
    "logging",
    "math",
    "multiprocessing",
    "os",
    "pathlib",
    "pickle",
    "platform",
    "pprint",
    "queue",
    "random",
    "re",
    "shlex",
    "shutil",
    "signal",
    "socket",
    "sqlite3",
    "ssl",
    "stat",
    "string",
    "struct",
    "subprocess",
    "sys",
    "tempfile",
    "textwrap",
    "threading",
    "time",
    "timeit",
    "tkinter",
    "token",
    "tokenize",
    "traceback",
    "types",
    "typing",
    "unittest",
    "urllib",
    "uuid",
    "warnings",
    "weakref",
    "xml",
    "zipfile",
    "zlib",
}

NAME_REGEX = re.compile(r"^[a-z0-9]+(-[a-z0-9]+)*$")
MAX_NAME_LEN = 64
MAX_DESC_LEN = 1024


def validate_skill_metadata(name: str, description: str) -> Tuple[bool, List[str]]:
    """Validates skill name and description against specification constraints."""
    errors = []

    if not name:
        errors.append("Skill name cannot be empty.")
    elif len(name) > MAX_NAME_LEN:
        errors.append(f"Skill name exceeds maximum length of {MAX_NAME_LEN} characters ({len(name)} chars).")
    elif not NAME_REGEX.match(name):
        errors.append(
            "Skill name must contain only lowercase alphanumeric characters and single hyphens "
            "(e.g. 'my-new-skill'). Cannot start or end with a hyphen."
        )

    if not description:
        errors.append("Skill description cannot be empty.")
    elif len(description) > MAX_DESC_LEN:
        errors.append(f"Skill description exceeds maximum length of {MAX_DESC_LEN} characters ({len(description)} chars).")

    return (len(errors) == 0, errors)


def parse_frontmatter(content: str) -> Tuple[Dict[str, str], str]:
    """Parses basic YAML frontmatter from SKILL.md without external dependencies."""
    if not content.startswith("---"):
        return {}, content

    parts = content.split("---", 2)
    if len(parts) < 3:
        return {}, content

    yaml_block = parts[1].strip()
    body = parts[2].strip()

    metadata = {}

    for line in yaml_block.splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        if ":" in line:
            key, val = line.split(":", 1)
            k = key.strip()
            v = val.strip().strip("\"'")
            metadata[k] = v

    return metadata, body


def scaffold_skill(
    name: str,
    description: str,
    target_dir: Path,
    skill_type: str = "simple",
    user_invoked: bool = False,
    is_complex: bool = False,  # Backwards compatibility
) -> Path:
    """Creates directory structure and boilerplate files for a new skill."""
    if is_complex and skill_type == "simple":
        skill_type = "complex"

    valid, errors = validate_skill_metadata(name, description)
    if not valid:
        raise ValueError("Invalid skill metadata:\n" + "\n".join(f"- {e}" for e in errors))

    skill_path = target_dir / name
    if skill_path.exists():
        raise FileExistsError(f"Skill directory already exists: {skill_path}")

    skill_path.mkdir(parents=True, exist_ok=True)

    title_words = [w.capitalize() for w in name.split("-")]
    skill_title = " ".join(title_words)
    skill_md = skill_path / "SKILL.md"

    frontmatter_lines = ["---", f"name: {name}", f"description: {description}"]
    if user_invoked:
        frontmatter_lines.append("disable-model-invocation: true")
    frontmatter_lines.append("---")
    frontmatter = "\n".join(frontmatter_lines)

    if skill_type == "router":
        content = f"""{frontmatter}

# {skill_title}

Router skill for dispatching task requests to specialized sub-skills or workflows.

## Sub-Skill Directory & Routing Map

| Intent / Task | Skill / Resource | Description |
| :--- | :--- | :--- |
| Task Category A | `sub-skill-a` | Description of when to use sub-skill A. |
| Task Category B | `sub-skill-b` | Description of when to use sub-skill B. |
| Task Category C | [reference-doc.md](references/reference-doc.md) | Reference documentation. |

## Guidance for Agent

1. Inspect the user request against the routing map above.
2. Direct the workflow to the appropriate sub-skill or reference file.
3. If no matching sub-skill applies, request clarification from the user.

## Completion Criteria

- [ ] Matched user request to target sub-skill or reference document cleanly.
- [ ] Direct routing advice provided to user.
"""
    elif skill_type == "complex":
        content = f"""{frontmatter}

# {skill_title}

Overview of {skill_title}.

## Workflow

1. **Initialization**: Read configuration or set up environment.
2. **Execution**: Run automation script:

   ```bash
   python3 skills/{name}/scripts/main.py --check
   ```

3. **Synthesis**: Process output and report results to user.

## Completion Criteria

- [ ] Automation scripts execute cleanly with exit code 0.
- [ ] Unit tests pass without error.
- [ ] Output complies with required schema.

## References

- [overview.md](references/overview.md) — Extended reference documentation.
"""
    else:  # simple
        content = f"""{frontmatter}

# {skill_title}

Overview of {skill_title}.

## Guidelines & Workflow

1. Step 1: Initial workflow step.
2. Step 2: Main task execution.
3. Step 3: Verification and output.

## Rules & Constraints

- Rule 1: Key guideline to follow.

## Completion Criteria

- [ ] Task execution completed without errors or unresolved exceptions.
- [ ] Output produced matches requested structure and parameters.
"""

    skill_md.write_text(content, encoding="utf-8")

    if skill_type == "complex":
        (skill_path / "references").mkdir(exist_ok=True)
        (skill_path / "templates").mkdir(exist_ok=True)
        (skill_path / "examples").mkdir(exist_ok=True)

        scripts_dir = skill_path / "scripts"
        tests_dir = scripts_dir / "tests"
        tests_dir.mkdir(parents=True, exist_ok=True)

        (skill_path / "references" / "overview.md").write_text(
            f"# {skill_title} Overview\n\nExtended reference documentation.\n", encoding="utf-8"
        )

        starter_script = scripts_dir / "main.py"
        starter_script.write_text(
            f"""#!/usr/bin/env python3
\"\"\"{skill_title} main automation script.\"\"\"

import argparse
import sys


def main() -> int:
    parser = argparse.ArgumentParser(description="{skill_title} CLI tool")
    parser.add_argument("--check", action="store_true", help="Run health check")
    args = parser.parse_args()

    print(f"{skill_title} script running. Check: {{args.check}}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
""",
            encoding="utf-8",
        )

        starter_test = tests_dir / "test_main.py"
        starter_test.write_text(
            f"""#!/usr/bin/env python3
\"\"\"Tests for {skill_title} main script.\"\"\"

import unittest


class TestMain(unittest.TestCase):
    def test_basic_pass(self) -> None:
        self.assertTrue(True)


if __name__ == "__main__":
    unittest.main()
""",
            encoding="utf-8",
        )

    return skill_path


def check_scope_creep(description: str) -> List[str]:
    """Detects potential scope creep (multiple compound task connectors in skill description)."""
    issues = []
    lower_desc = description.lower()

    compound_phrases = [" and also ", " as well as ", " along with ", " and additionally "]
    has_compound_phrase = any(phrase in lower_desc for phrase in compound_phrases)
    and_count = lower_desc.count(" and ")

    if has_compound_phrase or and_count >= 3:
        issues.append(
            f"Scope Creep / Slob warning: Skill description contains compound task connectors ('{description}'). "
            "Consider splitting into atomic skills."
        )
    return issues


def check_python_stdlib_compliance(scripts_dir: Path) -> List[str]:
    """Audits Python scripts to enforce ADR 0001 (Standard Library First)."""
    issues = []
    if not scripts_dir.is_dir():
        return issues

    for py_file in scripts_dir.rglob("*.py"):
        if "tests" in py_file.parts:
            continue
        try:
            tree = ast.parse(py_file.read_text(encoding="utf-8"), filename=str(py_file))
            for node in ast.walk(tree):
                imported_mods = []
                if isinstance(node, ast.Import):
                    for alias in node.names:
                        imported_mods.append(alias.name.split(".")[0])
                elif isinstance(node, ast.ImportFrom):
                    if node.module:
                        imported_mods.append(node.module.split(".")[0])

                for mod in imported_mods:
                    if mod and mod not in STDLIB_MODULES and not mod.startswith("."):
                        issues.append(f"Non-stdlib import detected in {py_file.name}: '{mod}' (violates ADR 0001).")
        except Exception as err:
            issues.append(f"Failed to parse AST for {py_file.name}: {err}")

    return issues


def validate_skill(skill_dir: Path) -> Tuple[bool, List[str]]:
    """Validates an existing skill directory against agentskills.io standard & design rules."""
    issues = []

    if not skill_dir.is_dir():
        return False, [f"Target path is not a directory: {skill_dir}"]

    skill_md = skill_dir / "SKILL.md"
    if not skill_md.exists():
        return False, [f"Missing required SKILL.md at: {skill_md}"]

    content = skill_md.read_text(encoding="utf-8")
    lines = content.splitlines()

    # Context Load Audit
    if len(lines) > 500:
        issues.append(f"Context Load warning: SKILL.md line count ({len(lines)}) exceeds limit of 500 lines.")

    metadata, body = parse_frontmatter(content)

    name = metadata.get("name", "")
    desc = metadata.get("description", "")

    valid_meta, meta_errs = validate_skill_metadata(name, desc)
    if not valid_meta:
        issues.extend(meta_errs)

    if desc:
        issues.extend(check_scope_creep(desc))

    if name and name != skill_dir.name:
        issues.append(f"Frontmatter name ('{name}') does not match directory name ('{skill_dir.name}').")

    # Audit Python script imports (ADR 0001)
    scripts_dir = skill_dir / "scripts"
    if scripts_dir.is_dir():
        issues.extend(check_python_stdlib_compliance(scripts_dir))

    # Completion Criteria Audit
    has_completion_criteria = (
        "## Completion Criteria" in body
        or "## Verification" in body
        or "## Verification Criteria" in body
        or "- [ ]" in body
        or "- [x]" in body
    )
    if not has_completion_criteria:
        issues.append(
            "Missing checkable completion criteria or verification section ('## Completion Criteria' or '## Verification')."
        )

    # Cognitive Load Audit (excessive nested decision branches without sub-skills/references)
    if body.count("if ") + body.count("else:") + body.count("elif ") > 12:
        if not (skill_dir / "references").is_dir() and not (skill_dir / "scripts").is_dir():
            issues.append(
                "High Cognitive Load: High volume of conditional branching in SKILL.md body. "
                "Consider pushing reference details to references/ or creating a sub-skill."
            )

    # Script Unit Tests Audit
    tests_dir = skill_dir / "scripts" / "tests"
    if tests_dir.is_dir():
        # Clear temporary module cache to prevent unittest loader collisions
        for mod_name in list(sys.modules.keys()):
            if mod_name.startswith("test_"):
                sys.modules.pop(mod_name, None)
        loader = unittest.TestLoader()
        suite = loader.discover(str(tests_dir))

        with open(os.devnull, "w") as null_stream:
            runner = unittest.TextTestRunner(stream=null_stream, verbosity=0)
            result = runner.run(suite)
        if not result.wasSuccessful():
            issues.append(
                f"Unit test suite in {tests_dir} failed ({len(result.failures)} failures, {len(result.errors)} errors)."
            )

    return (len(issues) == 0, issues)


def parse_args(args=None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Scaffold and validate Agent Skills")
    parser.add_argument("--name", type=str, help="Name of the skill to scaffold (e.g. 'my-skill')")
    parser.add_argument("--description", type=str, help="Description of when to trigger the skill")
    parser.add_argument("--target-dir", type=str, default="skills", help="Directory to scaffold skill inside")
    parser.add_argument(
        "--type",
        type=str,
        choices=["simple", "complex", "router"],
        default="simple",
        help="Type of skill template to scaffold",
    )
    parser.add_argument("--user-invoked", action="store_true", help="Mark skill as user-invoked (disable-model-invocation: true)")
    parser.add_argument("--complex", action="store_true", help="Alias for --type complex")
    parser.add_argument("--validate", type=str, help="Validate an existing skill directory")
    return parser.parse_args(args)


def main() -> int:
    args = parse_args()

    if args.validate:
        target_path = Path(args.validate).resolve()
        is_valid, issues = validate_skill(target_path)
        if is_valid:
            print(f"✅ Skill at '{target_path}' is VALID according to agentskills.io standard!")
            return 0
        else:
            print(f"❌ Skill validation FAILED for '{target_path}':")
            for issue in issues:
                print(f"  - {issue}")
            return 1

    if not args.name or not args.description:
        print("Error: Both --name and --description are required when scaffolding a skill.", file=sys.stderr)
        return 1

    target_dir = Path(args.target_dir).resolve()
    skill_type = "complex" if args.complex else args.type
    try:
        created_path = scaffold_skill(
            name=args.name,
            description=args.description,
            target_dir=target_dir,
            skill_type=skill_type,
            user_invoked=args.user_invoked,
        )
        print(f"🎉 Successfully scaffolded skill ({skill_type}) at: {created_path}")
        return 0
    except Exception as err:
        print(f"Error scaffolding skill: {err}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
