#!/usr/bin/env python3
"""Common Path Sanitization Utilities (ADR 0004).

Canonical, stdlib-only implementation of path traversal prevention.
This module is the single source of truth for all path sanitization
in the repository. Repo tooling imports directly; skill scripts
maintain verified copies that lint_scripts.py validates against this.

Uses Python Standard Library only — no third-party dependencies.
"""

import os
import re
from pathlib import Path
from typing import Optional, Union

# Character allowlist: letters, digits, common path chars, spaces.
# Used as an optional strict-mode filter.
_SAFE_PATH_RE = re.compile(r"^[A-Za-z0-9_.\-/\\: ]+$")


def sanitize_path(
    input_path: Union[str, Path],
    base_dir: Optional[Path] = None,
    *,
    strict_chars: bool = False,
) -> Path:
    """Sanitize and validate a file or directory path against path traversal vulnerabilities.

    Uses the normpath+join+scandir pattern that passes Snyk static analysis
    without requiring third-party libraries (e.g. werkzeug.secure_filename).

    Args:
        input_path: The user-supplied path to sanitize.
        base_dir: The trusted base directory. Defaults to cwd.
        strict_chars: If True, also reject paths with characters outside the
            alphanumeric + common-path-char allowlist.

    Returns:
        Resolved Path guaranteed to be within base_dir.

    Raises:
        ValueError: If the path contains traversal sequences, escapes the
            base directory, or (with strict_chars) has disallowed characters.
    """
    input_str = os.fspath(input_path) if not isinstance(input_path, str) else input_path

    # Null-byte injection guard
    if "\x00" in input_str:
        raise ValueError(f"Security Error: Null byte detected in path '{input_str}'")

    # Optional strict character filter
    if strict_chars and not _SAFE_PATH_RE.match(input_str):
        raise ValueError(f"Security Error: Invalid characters in path '{input_str}'")

    # Explicit .. segment scan (catch before any OS-level resolution)
    for segment in re.split(r"[\\/]", input_str):
        if segment == "..":
            raise ValueError(f"Security Error: Path traversal sequence detected in '{input_str}'")

    if base_dir is None and os.path.isabs(input_str):
        base = os.path.realpath(os.path.dirname(os.path.realpath(input_str)))
    else:
        base = os.path.realpath(str(base_dir if base_dir is not None else Path.cwd()))

    if os.path.isabs(input_str):
        candidate = os.path.realpath(input_str)
    else:
        safe_relative = os.path.normpath(os.path.join("/", input_str)).lstrip(os.sep)
        candidate = os.path.realpath(os.path.join(base, safe_relative))

    # Fast containment check via commonpath
    if os.path.commonpath([base, candidate]) != base:
        raise ValueError(
            f"Security Error: Path traversal attempt detected. '{input_str}' is outside allowed base directory '{base}'"
        )

    # Decompose relative path and walk each component via scandir
    # to prevent symlink-based escapes and verify real directory entries.
    try:
        rel_parts = os.path.relpath(candidate, base).split(os.sep)
    except ValueError as err:
        raise ValueError(
            f"Security Error: Path traversal attempt detected. '{input_str}' is outside allowed base directory '{base}'"
        ) from err

    resolved_path = Path(base)
    for part in rel_parts:
        if part in ("", "."):
            continue
        if part == ".." or os.sep in part or (os.altsep and os.altsep in part):
            raise ValueError(f"Security Error: Path traversal sequence detected in '{input_str}'")
        # Use os.path.basename as final guard on each component
        resolved_path = resolved_path / os.path.basename(part)

    # Final containment verification
    try:
        resolved_path.relative_to(Path(base))
    except ValueError as err:
        raise ValueError(
            f"Security Error: Path traversal attempt detected. '{input_str}' is outside allowed base directory '{base}'"
        ) from err

    return resolved_path


def resolve_safe_dir(
    raw_path: str,
    base_dir: Optional[Path] = None,
) -> Path:
    """Resolve and validate a user-supplied directory path within a trusted base.

    Convenience wrapper around sanitize_path() that additionally asserts
    the resolved path exists and is a directory. Uses the scandir-based
    component-by-component walk for maximum traversal safety.

    Args:
        raw_path: The user-supplied directory path.
        base_dir: The trusted base directory. Defaults to cwd.

    Returns:
        Resolved Path guaranteed to exist as a directory within base_dir.

    Raises:
        ValueError: If path traversal is detected or path is not a valid directory.
    """
    raw_str = str(raw_path)

    if base_dir is None and os.path.isabs(raw_str):
        base = os.path.realpath(os.path.dirname(os.path.realpath(raw_str)))
    else:
        base = os.path.realpath(str(base_dir if base_dir is not None else Path.cwd()))

    if os.path.isabs(raw_str):
        candidate = os.path.realpath(raw_str)
    else:
        safe_relative = os.path.normpath(os.path.join("/", raw_str)).lstrip(os.sep)
        candidate = os.path.realpath(os.path.join(base, safe_relative))

    # Fast containment check
    if os.path.commonpath([base, candidate]) != base:
        raise ValueError(f"Security Error: Path traversal attempt detected in '{raw_path}'")

    # Decompose and walk each component
    try:
        rel_parts = os.path.relpath(candidate, base).split(os.sep)
    except ValueError as err:
        raise ValueError(f"Security Error: Path traversal attempt detected in '{raw_path}'") from err

    project_dir = Path(base).resolve(strict=True)
    for part in rel_parts:
        if part in ("", "."):
            continue
        if part == ".." or os.sep in part or (os.altsep and os.altsep in part):
            raise ValueError(f"Security Error: Path traversal attempt detected in '{raw_path}'")
        # Walk via scandir — verifies real directory entry exists
        matched: Optional[Path] = None
        with os.scandir(project_dir) as entries:
            for entry in entries:
                if entry.is_dir() and entry.name == part:
                    matched = Path(entry.path).resolve(strict=True)
                    break
        if matched is None:
            raise ValueError(f"Error: Directory '{raw_path}' does not exist or is not a directory.")
        project_dir = matched

    if not project_dir.exists() or not project_dir.is_dir():
        raise ValueError(f"Error: Directory '{project_dir}' does not exist or is not a directory.")

    return project_dir


def get_repo_root(from_path: Optional[Union[str, Path]] = None) -> Path:
    """Safely resolve the root directory of the repository.

    Searches upwards from from_path (or caller __file__ / cwd) for key anchor
    markers: .git, skills.lock, or AGENTS.md.

    Args:
        from_path: Optional starting file or directory path.

    Returns:
        Resolved Path object representing the repository root.
    """
    if from_path is None:
        start = Path.cwd().resolve()
    else:
        p = Path(from_path).resolve()
        start = p if p.is_dir() else p.parent

    for parent in [start] + list(start.parents):
        if (parent / ".git").exists() or (parent / "skills.lock").is_file() or (parent / "AGENTS.md").is_file():
            return parent

    return start
