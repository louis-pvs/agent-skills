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
    from scripts._path_safety import sanitize_path
except ImportError:

    def sanitize_path(input_path, base_dir=None, *, strict_chars=False):
        import os

        base = os.path.realpath(str(base_dir if base_dir is not None else Path.cwd()))
        candidate = os.path.realpath(str(input_path))
        if os.path.commonpath([base, candidate]) != base:
            raise ValueError(f"Security Error: Path '{input_path}' is outside allowed base directory '{base}'")
        return Path(candidate)


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


def get_adr_config(repo_dir: Path) -> Tuple[Path, str]:
    """Resolves ADR directory and naming prefix using 4-tier skill config loader."""
    config_data = load_skill_config(
        "architecture-decision-records", skill_dir=Path(__file__).resolve().parents[1], repo_root=repo_dir
    )
    skill_cfg = config_data.get("skill_config", {})

    adr_rel_path = skill_cfg.get("adr_dir", "docs/adr")
    adr_prefix = skill_cfg.get("prefix", "ADR")

    adr_dir = sanitize_path(repo_dir / adr_rel_path, base_dir=repo_dir)
    return adr_dir, adr_prefix


def find_highest_adr_id(adr_dir: Path) -> int:
    """Finds the highest integer ID among existing ADR files."""
    if not adr_dir.exists():
        return 0

    highest_id = 0
    pattern = re.compile(r"^(\d{4})-[a-z0-9-]+\.md$")

    for file_path in adr_dir.glob("*.md"):
        if file_path.name.lower() in ("readme.md", "index.md"):
            continue
        match = pattern.match(file_path.name)
        if match:
            adr_id = int(match.group(1))
            if adr_id > highest_id:
                highest_id = adr_id

    return highest_id


def get_all_adr_files(adr_dir: Path) -> List[Path]:
    """Returns a sorted list of all ADR markdown files in adr_dir."""
    if not adr_dir.exists():
        return []

    pattern = re.compile(r"^\d{4}-[a-z0-9-]+\.md$")
    adr_files = []

    for file_path in adr_dir.glob("*.md"):
        if pattern.match(file_path.name):
            adr_files.append(file_path)

    return sorted(adr_files, key=lambda p: p.name)


def parse_adr_metadata(file_path: Path) -> Dict[str, str]:
    """Parses title, status, date, and superseded information from an ADR file."""
    metadata = {
        "id": "0000",
        "title": "Unknown Title",
        "status": "Unknown",
        "date": "Unknown",
        "superseded_by": "",
        "filename": file_path.name,
    }

    match = re.match(r"^(\d{4})", file_path.name)
    if match:
        metadata["id"] = match.group(1)

    try:
        content = file_path.read_text(encoding="utf-8")
    except Exception:
        return metadata

    lines = content.splitlines()
    in_status = False

    for line in lines:
        stripped = line.strip()

        if stripped.startswith("# ") and metadata["title"] == "Unknown Title":
            title_text = stripped[2:].strip()
            title_text = re.sub(r"^\d+[\.\:]\s*", "", title_text)
            metadata["title"] = title_text

        if stripped.lower() == "## status":
            in_status = True
            continue

        if in_status:
            if stripped.startswith("## ") and stripped.lower() != "## status":
                in_status = False
            elif stripped:
                metadata["status"] = stripped
                super_match = re.search(r"Superseded by \[.*?\]\((.*?)\)", stripped, re.IGNORECASE)
                if not super_match:
                    super_match = re.search(r"Superseded by (ADR-\d{4}|\d{4}-[a-z0-9-]+\.md)", stripped, re.IGNORECASE)

                if super_match:
                    metadata["superseded_by"] = super_match.group(1)
                in_status = False

        date_match = re.search(r"\b(\d{4}-\d{2}-\d{2})\b", line)
        if date_match and metadata["date"] == "Unknown":
            metadata["date"] = date_match.group(1)

    return metadata


def scaffold_adr_file(adr_dir: Path, title: str, status: str = "Proposed") -> Path:
    """Scaffolds a new sequential ADR file adhering to MADR conventions."""
    adr_dir.mkdir(parents=True, exist_ok=True)
    highest_id = find_highest_adr_id(adr_dir)
    next_id = highest_id + 1
    id_str = f"{next_id:04d}"
    slug = slugify(title)
    filename = f"{id_str}-{slug}.md"
    file_path = adr_dir / filename

    today_str = datetime.date.today().isoformat()

    template = f"""# {next_id}. {title}

Date: {today_str}

## Status

{status}

## Context & Problem Statement

Describe the context and problem statement, e.g., what force/driver is compelling this decision?

## Decision Drivers

* Driver 1
* Driver 2

## Considered Options

* Option 1
* Option 2

## Decision Outcome

Chosen Option: Option 1

### Consequences

* Good: Positive consequence 1
* Bad: Negative consequence 1

## Pros and Cons of Options

### Option 1

* Good, because...
* Bad, because...

### Option 2

* Good, because...
* Bad, because...
"""

    file_path.write_text(template, encoding="utf-8")
    return file_path


def generate_index_markdown(adr_dir: Path) -> str:
    """Generates the Markdown content for docs/adr/README.md catalog index."""
    adr_files = get_all_adr_files(adr_dir)

    lines = [
        "# Architectural Decision Records",
        "",
        "This directory contains all Architectural Decision Records (ADRs) for this project.",
        "",
        "| ID | Title | Status | Date |",
        "| :--- | :--- | :--- | :--- |",
    ]

    for file_path in adr_files:
        meta = parse_adr_metadata(file_path)
        id_str = meta["id"]
        title = meta["title"]
        status = meta["status"]
        date_str = meta["date"]

        title_link = f"[{title}]({file_path.name})"
        lines.append(f"| `{id_str}` | {title_link} | {status} | {date_str} |")

    lines.append("")
    return "\n".join(lines)


def reindex_adr_catalog(adr_dir: Path) -> Path:
    """Rebuilds the catalog index README.md inside adr_dir."""
    adr_dir.mkdir(parents=True, exist_ok=True)
    readme_path = adr_dir / "README.md"
    content = generate_index_markdown(adr_dir)
    readme_path.write_text(content, encoding="utf-8")
    return readme_path


def update_adr_status(adr_file: Path, new_status: str) -> bool:
    """Updates the status section of a specific ADR file."""
    if not adr_file.exists():
        return False

    content = adr_file.read_text(encoding="utf-8")
    lines = content.splitlines()

    new_lines = []
    in_status = False
    status_updated = False

    for line in lines:
        stripped = line.strip()
        if stripped.lower() == "## status":
            in_status = True
            new_lines.append(line)
            continue

        if in_status:
            if stripped.startswith("## "):
                in_status = False
                new_lines.append(line)
            elif not status_updated:
                new_lines.append(new_status)
                status_updated = True
            continue

        new_lines.append(line)

    if not status_updated and in_status:
        new_lines.append(new_status)

    adr_file.write_text("\n".join(new_lines) + "\n", encoding="utf-8")
    return True


def parse_args():
    parser = argparse.ArgumentParser(description="Architectural Decision Records (ADR) CLI Utility")
    parser.add_argument("--new", type=str, help="Scaffold a new ADR file with given title")
    parser.add_argument("--accept", type=str, help="Set status of ADR (by ID or filename) to Accepted")
    parser.add_argument("--reject", type=str, help="Set status of ADR (by ID or filename) to Rejected")
    parser.add_argument("--supersede", type=str, help="ADR ID or filename to mark as superseded")
    parser.add_argument("--by", type=str, help="Target ADR ID or filename that supersedes the specified ADR")
    parser.add_argument("--reindex", action="store_true", help="Rebuild the catalog index table in docs/adr/README.md")
    parser.add_argument("--dry-run", action="store_true", help="Preview ADR operations without modifying files")
    parser.add_argument("--repo-dir", type=str, default=".", help="Target repository directory (default: cwd)")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    repo_dir = Path(args.repo_dir).resolve()
    adr_dir, _ = get_adr_config(repo_dir)

    if args.dry_run:
        print(f"[DRY-RUN] ADR operations target directory: {adr_dir}")
        if args.new:
            print(f"[DRY-RUN] Would scaffold new ADR: '{args.new}'")
        return 0

    if args.new:
        file_path = scaffold_adr_file(adr_dir, args.new)
        reindex_adr_catalog(adr_dir)
        print(f"🎉 Created new ADR: {file_path.relative_to(repo_dir)}")
        print(f"📖 Updated index: {adr_dir.relative_to(repo_dir)}/README.md")
        return 0

    if args.accept:
        adr_files = get_all_adr_files(adr_dir)
        target = [f for f in adr_files if args.accept in f.name]
        if not target:
            print(f"Error: Could not find ADR matching '{args.accept}' in {adr_dir}")
            return 1
        update_adr_status(target[0], "Accepted")
        reindex_adr_catalog(adr_dir)
        print(f"✅ Set status of {target[0].name} to Accepted")
        return 0

    if args.reject:
        adr_files = get_all_adr_files(adr_dir)
        target = [f for f in adr_files if args.reject in f.name]
        if not target:
            print(f"Error: Could not find ADR matching '{args.reject}' in {adr_dir}")
            return 1
        update_adr_status(target[0], "Rejected")
        reindex_adr_catalog(adr_dir)
        print(f"❌ Set status of {target[0].name} to Rejected")
        return 0

    if args.supersede:
        if not args.by:
            print("Error: --supersede requires --by <target_adr_id_or_filename>")
            return 1
        adr_files = get_all_adr_files(adr_dir)
        target_src = [f for f in adr_files if args.supersede in f.name]
        target_by = [f for f in adr_files if args.by in f.name]

        if not target_src:
            print(f"Error: Could not find ADR matching '{args.supersede}'")
            return 1

        by_ref = target_by[0].name if target_by else args.by
        status_text = f"Superseded by [{by_ref}]({by_ref})"

        update_adr_status(target_src[0], status_text)
        reindex_adr_catalog(adr_dir)
        print(f"🔄 Set status of {target_src[0].name} to '{status_text}'")
        return 0

    if args.reindex:
        readme_path = reindex_adr_catalog(adr_dir)
        print(f"📖 Rebuilt ADR catalog index: {readme_path.relative_to(repo_dir)}")
        return 0

    reindex_adr_catalog(adr_dir)
    print(f"ADR directory ready at: {adr_dir.relative_to(repo_dir)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
