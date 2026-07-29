#!/usr/bin/env python3
"""Doc Auditor script for technical documentation validation.

Audits Markdown files for GitHub Flavored Markdown (GFM) compliance:
- Ensures exactly one H1 tag per file (outside code blocks).
- Validates GFM alert callout formatting.
- Flags ASCII art inside code blocks (recommends Mermaid diagrams).
- Performs privacy checks (no hardcoded absolute user paths).
"""

import argparse
import re
import sys
from pathlib import Path
from typing import List, Tuple


def audit_markdown_file(file_path: Path) -> Tuple[bool, List[str]]:
    """Audit a single markdown file for GFM and quality rules.

    Returns:
        Tuple[bool, List[str]]: (passed, list_of_finding_messages)
    """
    if not file_path.exists():
        return False, [f"File not found: {file_path}"]

    try:
        content = file_path.read_text(encoding="utf-8")
    except Exception as exc:
        return False, [f"Failed to read file {file_path}: {exc}"]

    lines = content.splitlines()
    findings: List[str] = []
    is_valid = True

    in_code_block = False
    code_block_fence_len = 0
    code_block_lang = ""

    h1_matches: List[Tuple[int, str]] = []
    ascii_art_patterns = [
        r"\+---\+",
        r"\|   \|",
        r"/\s*\\\s*/",
        r"\[\s*\]--->",
        r"<===>",
    ]

    privacy_pattern = re.compile(r"(/home/[a-zA-Z0-9_-]+|C:\\Users\\[a-zA-Z0-9_-]+)")
    invalid_alert_pattern = re.compile(r"^>\s*\[!(.*?)\]")
    valid_alerts = {"NOTE", "TIP", "IMPORTANT", "WARNING", "CAUTION"}

    for idx, line in enumerate(lines, start=1):
        stripped = line.strip()

        # Handle fenced code block toggling (3 or more backticks)
        fence_match = re.match(r"^(~{3,}|`{3,})(.*)$", stripped)
        if fence_match:
            fence_chars = fence_match.group(1)
            fence_len = len(fence_chars)
            if not in_code_block:
                in_code_block = True
                code_block_fence_len = fence_len
                code_block_lang = fence_match.group(2).strip().lower()
            elif in_code_block and fence_len >= code_block_fence_len:
                in_code_block = False
                code_block_fence_len = 0
                code_block_lang = ""
            continue

        # 1. H1 Count Check (only outside code blocks)
        if not in_code_block:
            if stripped.startswith("# ") and not stripped.startswith("##"):
                h1_matches.append((idx, line))

        # 2. ASCII Art Check inside code blocks
        if in_code_block and code_block_lang not in ("mermaid", "json", "yaml", "toml", "bash", "sh"):
            for pattern in ascii_art_patterns:
                if re.search(pattern, line):
                    findings.append(f"WARNING: Line {idx} appears to contain ASCII art. Use 'mermaid' diagrams instead.")
                    break

        # 3. Privacy Check for hardcoded user paths
        match = privacy_pattern.search(line)
        if match:
            matched_path = match.group(0)
            if not ("username" in matched_path.lower() or "user" in matched_path.lower() or "path/to" in line):
                is_valid = False
                findings.append(
                    f"ERROR: Line {idx} contains hardcoded personal user path ('{matched_path}'). Use relative or generic paths."
                )

        # 4. GFM Alert Callout Check (only outside code blocks)
        if not in_code_block:
            alert_match = invalid_alert_pattern.search(stripped)
            if alert_match:
                tag = alert_match.group(1).upper()
                if tag not in valid_alerts:
                    is_valid = False
                    valid_str = ", ".join(sorted(valid_alerts))
                    findings.append(f"ERROR: Line {idx} has invalid GFM alert tag '[!{tag}]'. Must be one of {valid_str}.")

    # Evaluate H1 findings after scanning file
    if len(h1_matches) == 0:
        findings.append("WARNING: No H1 title ('# Title') found in document.")
    elif len(h1_matches) > 1:
        is_valid = False
        lines_str = ", ".join(str(item[0]) for item in h1_matches)
        findings.append(f"ERROR: Multiple H1 titles found on lines [{lines_str}]. GFM requires exactly one H1 per document.")

    return is_valid, findings


def main() -> int:
    parser = argparse.ArgumentParser(description="Audit Markdown files against GFM rules and quality directives.")
    parser.add_argument("paths", nargs="+", type=Path, help="Paths to Markdown file(s) to audit.")

    args = parser.parse_args()

    overall_success = True
    for path in args.paths:
        print(f"Auditing: {path}")
        passed, findings = audit_markdown_file(path)

        if findings:
            for finding in findings:
                print(f"  - {finding}")
        else:
            print("  ✓ All checks passed cleanly.")

        if not passed:
            overall_success = False

    return 0 if overall_success else 1


if __name__ == "__main__":
    sys.exit(main())
