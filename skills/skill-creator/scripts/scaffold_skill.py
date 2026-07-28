#!/usr/bin/env python3
"""Skill Scaffold and Validator Utility.

Automates the creation and validation of Agent Skills according to agentskills.io standard.
Uses Python Standard Library only.
"""

import argparse
import os
import re
import sys
import unittest
from pathlib import Path
from typing import Dict, List, Tuple

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
    current_key = None

    for line in yaml_block.splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        if ":" in line:
            key, val = line.split(":", 1)
            current_key = key.strip()
            metadata[current_key] = val.strip().strip('"\'')

    return metadata, body


def scaffold_skill(
    name: str,
    description: str,
    target_dir: Path,
    is_complex: bool = False,
) -> Path:
    """Creates directory structure and boilerplate files for a new skill."""
    valid, errors = validate_skill_metadata(name, description)
    if not valid:
        raise ValueError("Invalid skill metadata:\n" + "\n".join(f"- {e}" for e in errors))

    skill_path = target_dir / name
    if skill_path.exists():
        raise FileExistsError(f"Skill directory already exists: {skill_path}")

    skill_path.mkdir(parents=True, exist_ok=True)

    # Generate SKILL.md
    skill_md = skill_path / "SKILL.md"
    title_words = [w.capitalize() for w in name.split("-")]
    skill_title = " ".join(title_words)

    content = f"""---
name: {name}
description: {description}
---

# {skill_title}

Overview of {skill_title}.

## Guidelines & Workflow

1. Step 1: Initial workflow step.
2. Step 2: Main task execution.
3. Step 3: Verification and output.

## Rules & Constraints

- Rule 1: Key guideline to follow.

## Verification

Run verification tests or manual checks to confirm completion.
"""
    skill_md.write_text(content, encoding="utf-8")

    if is_complex:
        (skill_path / "references").mkdir(exist_ok=True)
        (skill_path / "templates").mkdir(exist_ok=True)
        (skill_path / "examples").mkdir(exist_ok=True)
        
        scripts_dir = skill_path / "scripts"
        tests_dir = scripts_dir / "tests"
        tests_dir.mkdir(parents=True, exist_ok=True)

        # Starter reference file
        (skill_path / "references" / "overview.md").write_text(
            f"# {skill_title} Overview\n\nExtended reference documentation.\n",
            encoding="utf-8"
        )

        # Starter script
        starter_script = scripts_dir / "main.py"
        starter_script.write_text(
            f"""#!/usr/bin/env python3
\"\"\"{skill_title} main automation script.\"\"\"

import argparse
import sys
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser(description="{skill_title} CLI tool")
    parser.add_argument("--check", action="store_true", help="Run health check")
    args = parser.parse_args()

    print(f"{skill_title} script running. Check: {{args.check}}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
""",
            encoding="utf-8"
        )

        # Starter test
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
            encoding="utf-8"
        )

    return skill_path


def validate_skill(skill_dir: Path) -> Tuple[bool, List[str]]:
    """Validates an existing skill directory against agentskills.io standard."""
    issues = []

    if not skill_dir.is_dir():
        return False, [f"Target path is not a directory: {skill_dir}"]

    skill_md = skill_dir / "SKILL.md"
    if not skill_md.exists():
        return False, [f"Missing required SKILL.md at: {skill_md}"]

    content = skill_md.read_text(encoding="utf-8")
    lines = content.splitlines()

    if len(lines) > 500:
        issues.append(f"SKILL.md line count ({len(lines)}) exceeds recommended limit of 500 lines.")

    metadata, body = parse_frontmatter(content)

    name = metadata.get("name", "")
    desc = metadata.get("description", "")

    valid_meta, meta_errs = validate_skill_metadata(name, desc)
    if not valid_meta:
        issues.extend(meta_errs)

    if name and name != skill_dir.name:
        issues.append(f"Frontmatter name ('{name}') does not match directory name ('{skill_dir.name}').")

    # Run script unit tests if tests directory exists
    tests_dir = skill_dir / "scripts" / "tests"
    if tests_dir.is_dir():
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
    parser.add_argument("--complex", action="store_true", help="Create subfolders (references/, scripts/, templates/)")
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
    try:
        created_path = scaffold_skill(args.name, args.description, target_dir, is_complex=args.complex)
        print(f"🎉 Successfully scaffolded skill at: {created_path}")
        return 0
    except Exception as err:
        print(f"Error scaffolding skill: {err}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
