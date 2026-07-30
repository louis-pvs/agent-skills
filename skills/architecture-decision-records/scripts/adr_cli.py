#!/usr/bin/env python3
"""CLI utility for managing Architectural Decision Records (ADRs).

Supports initializing ADR repos, generating next-sequential ADR files,
superseding previous decisions, auto-generating index tables, and validating ADR format integrity.
"""

import argparse
import datetime
import re
import sys
from pathlib import Path
from typing import Dict, List, Optional, Tuple

# Dynamic sys.path Root Resolution
repo_root = Path(__file__).resolve().parents[3]
if str(repo_root) not in sys.path:
    sys.path.insert(0, str(repo_root))

try:
    from scripts._config_safety import load_skill_config
except ImportError:

    def load_skill_config(skill_name: str, skill_dir: Optional[Path] = None, repo_root: Optional[Path] = None):
        return {"repo_config": {}, "skill_config": {}}


def slugify(text: str) -> str:
    """Converts text into a clean kebab-case URL/filename slug."""
    text = text.lower().strip()
    text = re.sub(r"[^\w\s-]", "", text)
    text = re.sub(r"[\s_-]+", "-", text)
    return text.strip("-")


def get_adr_config(target_dir: Optional[str] = None) -> Dict[str, str]:
    """Loads configuration with defaults following ADR 0005 hierarchy."""
    cfg = load_skill_config("architecture-decision-records")
    skill_section = cfg.get("skill_config", {}).get("architecture_decision_records", {})

    adr_dir = target_dir or skill_section.get("adr_dir", "docs/adr")
    template = skill_section.get("default_template", "madr")
    id_padding = int(skill_section.get("id_padding", 4))

    return {
        "adr_dir": adr_dir,
        "template": template,
        "id_padding": str(id_padding),
    }


def find_highest_adr_id(adr_path: Path) -> int:
    """Finds highest integer ID prefix among existing ADR files in directory."""
    if not adr_path.exists():
        return -1

    highest = -1
    for file in adr_path.glob("*.md"):
        if file.name.lower() == "readme.md":
            continue
        match = re.match(r"^(\d+)-", file.name)
        if match:
            val = int(match.group(1))
            if val > highest:
                highest = val
    return highest


def get_all_adr_files(adr_path: Path) -> List[Path]:
    """Returns sorted list of ADR files excluding README.md."""
    if not adr_path.exists():
        return []

    files = []
    for file in adr_path.glob("*.md"):
        if file.name.lower() == "readme.md":
            continue
        if re.match(r"^\d+-", file.name):
            files.append(file)

    files.sort(key=lambda p: int(re.match(r"^(\d+)-", p.name).group(1)))  # type: ignore[union-attr]
    return files


def parse_adr_metadata(adr_file: Path) -> Dict[str, str]:
    """Extracts title, status, date, and deciders from an ADR file."""
    content = adr_file.read_text(encoding="utf-8")
    lines = content.splitlines()

    title = adr_file.stem
    status = "Unknown"
    date_str = ""

    # Extract H1 title
    for line in lines:
        if line.startswith("# "):
            title = line[2:].strip()
            break

    # Extract Status and Date metadata lines
    for line in lines:
        line_clean = line.strip()
        if line_clean.startswith("* Status:") or line_clean.startswith("Status:"):
            status = line_clean.split(":", 1)[1].strip()
        elif line_clean.startswith("* Date:") or line_clean.startswith("Date:"):
            date_str = line_clean.split(":", 1)[1].strip()

    return {
        "filename": adr_file.name,
        "title": title,
        "status": status,
        "date": date_str,
    }


def generate_index_markdown(adr_path: Path) -> str:
    """Generates Markdown index content for docs/adr/README.md."""
    adr_files = get_all_adr_files(adr_path)

    lines = [
        "# Architectural Decision Records (ADRs)",
        "",
        "This directory contains architectural decision records for this project.",
        "",
        "| ID | Title | Status | Date |",
        "| :--- | :--- | :--- | :--- |",
    ]

    for file in adr_files:
        meta = parse_adr_metadata(file)
        match = re.match(r"^(\d+)-", file.name)
        adr_id = match.group(1) if match else "0000"
        title_link = f"[{meta['title']}]({meta['filename']})"
        lines.append(f"| {adr_id} | {title_link} | {meta['status']} | {meta['date']} |")

    lines.append("")
    return "\n".join(lines)


def init_adr_repo(adr_dir_str: str, dry_run: bool = False) -> Path:
    """Scaffolds ADR directory, initial ADR 0000, and README index."""
    adr_path = Path(adr_dir_str)
    if dry_run:
        print(f"[DRY-RUN] Would initialize ADR repository at {adr_path}")
        return adr_path

    adr_path.mkdir(parents=True, exist_ok=True)

    adr_0000 = adr_path / "0000-use-markdown-architectural-decision-records.md"
    if not adr_0000.exists():
        today = datetime.date.today().isoformat()
        content = f"""# 0. Use Markdown Architectural Decision Records

* Status: Accepted
* Date: {today}

## Context

We need to record architectural decisions made on this project so future
team members and maintainers understand the rationale and context behind major design choices.

## Decision

We will use Markdown Architectural Decision Records (MADR) stored in `{adr_dir_str}/` to log architecture decisions.

## Consequences

* Architectural choices are documented near source code in version control.
* Decisions become traceable, immutable, and easy to review during code audits.
"""
        adr_0000.write_text(content, encoding="utf-8")
        print(f"Created {adr_0000}")

    index_file = adr_path / "README.md"
    index_file.write_text(generate_index_markdown(adr_path), encoding="utf-8")
    print(f"Updated index {index_file}")

    return adr_path


def new_adr(
    title: str,
    adr_dir_str: str,
    template_type: str = "madr",
    padding: int = 4,
    dry_run: bool = False,
) -> Path:
    """Generates the next sequential ADR file from specified template."""
    adr_path = Path(adr_dir_str)

    highest_id = find_highest_adr_id(adr_path)
    next_id = highest_id + 1 if highest_id >= 0 else 0
    formatted_id = f"{next_id:0{padding}d}"

    slug = slugify(title)
    filename = f"{formatted_id}-{slug}.md"
    target_file = adr_path / filename

    if dry_run:
        print(f"[DRY-RUN] Would create ADR file {target_file} (template: {template_type})")
        return target_file

    adr_path.mkdir(parents=True, exist_ok=True)

    if target_file.exists():
        raise FileExistsError(f"ADR file {target_file} already exists.")

    today = datetime.date.today().isoformat()

    # Find template file
    skill_dir = Path(__file__).resolve().parents[1]
    template_file = skill_dir / "templates" / f"{template_type}.md"
    if not template_file.exists():
        template_file = skill_dir / "templates" / "madr.md"

    template_content = template_file.read_text(encoding="utf-8")

    # Replace placeholders
    content = template_content.replace("{TITLE}", f"{next_id}. {title}")
    content = content.replace("{STATUS}", "Proposed")
    content = content.replace("{DATE}", today)
    content = content.replace("{DECIDERS}", "Development Team")
    content = content.replace("{TECHNICAL_STORY}", "N/A")
    content = content.replace("{CONTEXT}", "Describe the context and problem statement here.")
    content = content.replace("{DRIVER_1}", "Quality requirement 1")
    content = content.replace("{DRIVER_2}", "Constraint 1")
    content = content.replace("{OPTION_1}", "Option 1")
    content = content.replace("{OPTION_2}", "Option 2")
    content = content.replace("{CHOSEN_OPTION}", "Option 1")
    content = content.replace("{REASON}", "it fulfills core requirements best")
    content = content.replace("{PRO_1}", "improves maintainability")
    content = content.replace("{CON_1}", "requires initial migration effort")
    content = content.replace("{PRO}", "positive outcome")
    content = content.replace("{CON}", "negative outcome")
    content = content.replace("{DECISION}", "State the chosen decision here.")
    content = content.replace("{CONSEQUENCES}", "Describe trade-offs and consequences here.")

    target_file.write_text(content, encoding="utf-8")
    print(f"Created ADR {target_file}")

    # Auto-reindex
    index_file = adr_path / "README.md"
    index_file.write_text(generate_index_markdown(adr_path), encoding="utf-8")
    print(f"Updated index {index_file}")

    return target_file


def supersede_adr(old_id: int, new_id: int, adr_dir_str: str, dry_run: bool = False) -> Tuple[Path, Path]:
    """Updates old ADR status to Superseded by new ADR ID."""
    adr_path = Path(adr_dir_str)
    adr_files = get_all_adr_files(adr_path)

    old_file: Optional[Path] = None
    new_file: Optional[Path] = None

    for file in adr_files:
        match = re.match(r"^(\d+)-", file.name)
        if match:
            val = int(match.group(1))
            if val == old_id:
                old_file = file
            elif val == new_id:
                new_file = file

    if not old_file:
        raise FileNotFoundError(f"ADR with ID {old_id} not found in {adr_dir_str}")
    if not new_file:
        raise FileNotFoundError(f"ADR with ID {new_id} not found in {adr_dir_str}")

    if dry_run:
        print(f"[DRY-RUN] Would update status of {old_file.name} to 'Superseded by {new_file.name}'")
        return old_file, new_file

    # Update old ADR status
    old_content = old_file.read_text(encoding="utf-8")
    new_status_line = f"* Status: Superseded by [{new_file.stem}]({new_file.name})"

    if re.search(r"^\*\s*Status:.*$", old_content, flags=re.MULTILINE):
        old_content = re.sub(r"^\*\s*Status:.*$", new_status_line, old_content, flags=re.MULTILINE)
    elif re.search(r"^Status:.*$", old_content, flags=re.MULTILINE):
        old_content = re.sub(r"^Status:.*$", new_status_line, old_content, flags=re.MULTILINE)
    else:
        lines = old_content.splitlines()
        lines.insert(2, new_status_line)
        old_content = "\n".join(lines)

    old_file.write_text(old_content, encoding="utf-8")
    print(f"Updated {old_file.name} status to Superseded by {new_file.name}")

    # Reindex
    index_file = adr_path / "README.md"
    index_file.write_text(generate_index_markdown(adr_path), encoding="utf-8")

    return old_file, new_file


def validate_adrs(adr_dir_str: str) -> Dict[str, List[str]]:
    """Validates ADR directory integrity, links, and ID sequences."""
    adr_path = Path(adr_dir_str)
    errors: List[str] = []
    warnings: List[str] = []

    if not adr_path.exists():
        errors.append(f"ADR directory '{adr_dir_str}' does not exist.")
        return {"errors": errors, "warnings": warnings}

    adr_files = get_all_adr_files(adr_path)
    if not adr_files:
        warnings.append(f"No ADR files found in '{adr_dir_str}'.")
        return {"errors": errors, "warnings": warnings}

    expected_id = 0
    file_map: Dict[int, Path] = {}

    for file in adr_files:
        match = re.match(r"^(\d+)-", file.name)
        if not match:
            errors.append(f"File '{file.name}' does not match expected ID pattern '000X-name.md'.")
            continue

        adr_id = int(match.group(1))
        file_map[adr_id] = file

        if adr_id != expected_id:
            warnings.append(f"ID sequence jump: expected ADR ID {expected_id}, but found {adr_id} ({file.name}).")
            expected_id = adr_id

        expected_id += 1

        # Check metadata
        meta = parse_adr_metadata(file)
        if not meta["title"]:
            errors.append(f"File '{file.name}' missing H1 title.")
        if meta["status"] == "Unknown":
            warnings.append(f"File '{file.name}' missing explicit Status line.")

    # Check index existence
    index_file = adr_path / "README.md"
    if not index_file.exists():
        warnings.append(f"Index file '{index_file}' is missing.")

    return {"errors": errors, "warnings": warnings}


def main():
    parser = argparse.ArgumentParser(description="Architectural Decision Record (ADR) CLI Tool")
    parser.add_argument("--adr-dir", type=str, help="Override ADR target directory")
    parser.add_argument("--template", type=str, choices=["madr", "nygard"], default="madr", help="Template format")
    parser.add_argument("--init", action="store_true", help="Initialize ADR directory and index")
    parser.add_argument("--new", type=str, help="Create a new ADR with specified title")
    parser.add_argument("--supersede", nargs=2, type=int, metavar=("OLD_ID", "NEW_ID"), help="Supersede OLD_ID with NEW_ID")
    parser.add_argument("--reindex", action="store_true", help="Rebuild ADR README.md index table")
    parser.add_argument("--validate", action="store_true", help="Validate ADR files integrity")
    parser.add_argument("--dry-run", action="store_true", help="Preview actions without modifying files")

    args = parser.parse_args()

    cfg = get_adr_config(args.adr_dir)
    target_dir = cfg["adr_dir"]
    template_type = args.template or cfg["template"]

    if args.init:
        init_adr_repo(target_dir, dry_run=args.dry_run)
    elif args.new:
        new_adr(args.new, target_dir, template_type=template_type, dry_run=args.dry_run)
    elif args.supersede:
        supersede_adr(args.supersede[0], args.supersede[1], target_dir, dry_run=args.dry_run)
    elif args.reindex:
        adr_path = Path(target_dir)
        if args.dry_run:
            print(f"[DRY-RUN] Would reindex {adr_path / 'README.md'}")
        else:
            index_file = adr_path / "README.md"
            index_file.write_text(generate_index_markdown(adr_path), encoding="utf-8")
            print(f"Reindexed {index_file}")
    elif args.validate:
        res = validate_adrs(target_dir)
        print("=== ADR Validation Results ===")
        if res["errors"]:
            print(f"❌ Errors ({len(res['errors'])}):")
            for e in res["errors"]:
                print(f"  - {e}")
        else:
            print("✅ No errors found.")

        if res["warnings"]:
            print(f"⚠️ Warnings ({len(res['warnings'])}):")
            for w in res["warnings"]:
                print(f"  - {w}")
        if res["errors"]:
            sys.exit(1)
    else:
        parser.print_help()


if __name__ == "__main__":
    main()
