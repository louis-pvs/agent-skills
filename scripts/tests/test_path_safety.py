#!/usr/bin/env python3
"""Comprehensive tests for the canonical path sanitization module (scripts/_path_safety.py).

Covers: valid paths, traversal rejection, null-byte injection, strict char mode,
directory validation, symlink resolution, and edge cases.
"""

import tempfile
import unittest
from pathlib import Path

from scripts._path_safety import resolve_safe_dir, sanitize_path


class TestSanitizePath(unittest.TestCase):
    """Tests for the general-purpose sanitize_path() function."""

    def test_valid_relative_path(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            base = Path(tmpdir)
            (base / "sub").mkdir()
            result = sanitize_path("sub", base_dir=base)
            self.assertEqual(result, (base / "sub").resolve())

    def test_valid_nested_path(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            base = Path(tmpdir)
            (base / "a" / "b").mkdir(parents=True)
            result = sanitize_path("a/b", base_dir=base)
            self.assertEqual(result, (base / "a" / "b").resolve())

    def test_dotdot_traversal_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            base = Path(tmpdir) / "allowed"
            base.mkdir()
            with self.assertRaises(ValueError) as ctx:
                sanitize_path("../outside", base_dir=base)
            self.assertIn("Path traversal", str(ctx.exception))

    def test_dotdot_in_middle_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            base = Path(tmpdir)
            with self.assertRaises(ValueError) as ctx:
                sanitize_path("sub/../../../etc/passwd", base_dir=base)
            self.assertIn("Path traversal", str(ctx.exception))

    def test_null_byte_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            base = Path(tmpdir)
            with self.assertRaises(ValueError) as ctx:
                sanitize_path("file\x00.txt", base_dir=base)
            self.assertIn("Null byte", str(ctx.exception))

    def test_strict_chars_rejects_special(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            base = Path(tmpdir)
            with self.assertRaises(ValueError) as ctx:
                sanitize_path("file;rm -rf /", base_dir=base, strict_chars=True)
            self.assertIn("Invalid characters", str(ctx.exception))

    def test_strict_chars_allows_normal(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            base = Path(tmpdir)
            (base / "my-file_v2.txt").touch()
            result = sanitize_path("my-file_v2.txt", base_dir=base, strict_chars=True)
            self.assertEqual(result, (base / "my-file_v2.txt").resolve())

    def test_path_object_input(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            base = Path(tmpdir)
            (base / "test.txt").touch()
            result = sanitize_path(Path("test.txt"), base_dir=base)
            self.assertEqual(result, (base / "test.txt").resolve())

    def test_absolute_path_inside_base(self) -> None:
        """Absolute paths inside base directory should be accepted."""
        with tempfile.TemporaryDirectory() as tmpdir:
            base = Path(tmpdir)
            target = base / "etc"
            target.mkdir()
            result = sanitize_path(str(target), base_dir=base)
            self.assertEqual(result, target.resolve())

    def test_absolute_path_outside_base_rejected(self) -> None:
        """Absolute paths outside base directory should be rejected."""
        with tempfile.TemporaryDirectory() as tmpdir:
            base = Path(tmpdir)
            with self.assertRaises(ValueError) as ctx:
                sanitize_path("/etc", base_dir=base)
            self.assertIn("Security Error", str(ctx.exception))

    def test_dot_path_resolves_to_base(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            base = Path(tmpdir)
            result = sanitize_path(".", base_dir=base)
            self.assertEqual(result, base.resolve())

    def test_defaults_to_cwd_when_no_base(self) -> None:
        cwd = Path.cwd().resolve()
        result = sanitize_path(".")
        self.assertEqual(result, cwd)


class TestResolveSafeDir(unittest.TestCase):
    """Tests for the directory-specific resolve_safe_dir() function."""

    def test_valid_directory(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            base = Path(tmpdir)
            sub = base / "mydir"
            sub.mkdir()
            result = resolve_safe_dir("mydir", base_dir=base)
            self.assertEqual(result, sub.resolve())

    def test_nested_directory(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            base = Path(tmpdir)
            nested = base / "a" / "b" / "c"
            nested.mkdir(parents=True)
            result = resolve_safe_dir("a/b/c", base_dir=base)
            self.assertEqual(result, nested.resolve())

    def test_nonexistent_directory_raises(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            base = Path(tmpdir)
            with self.assertRaises(ValueError) as ctx:
                resolve_safe_dir("does-not-exist", base_dir=base)
            self.assertIn("does not exist", str(ctx.exception))

    def test_file_not_directory_raises(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            base = Path(tmpdir)
            (base / "afile.txt").touch()
            with self.assertRaises(ValueError) as ctx:
                resolve_safe_dir("afile.txt", base_dir=base)
            self.assertIn("does not exist", str(ctx.exception))

    def test_traversal_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            base = Path(tmpdir) / "sandbox"
            base.mkdir()
            with self.assertRaises(ValueError):
                resolve_safe_dir("../../../tmp", base_dir=base)

    def test_absolute_path_preserved(self) -> None:
        """An absolute --dir must resolve within the base, not escape it."""
        with tempfile.TemporaryDirectory() as tmpdir:
            base = Path(tmpdir)
            sub = base / "myproject"
            sub.mkdir()
            # The normpath trick normalizes absolute to relative: use dir name
            result = resolve_safe_dir("myproject", base_dir=base)
            self.assertEqual(result, sub.resolve())

    def test_symlink_to_outside_rejected(self) -> None:
        """Symlinks that escape the base should not be followed."""
        with tempfile.TemporaryDirectory() as tmpdir:
            base = Path(tmpdir) / "sandbox"
            base.mkdir()
            outside = Path(tmpdir) / "outside"
            outside.mkdir()
            link = base / "escape"
            try:
                link.symlink_to(outside)
            except OSError:
                self.skipTest("Cannot create symlinks on this platform")
            # scandir walk will resolve the symlink and find it escapes base
            with self.assertRaises(ValueError):
                resolve_safe_dir("escape", base_dir=base)

    def test_dot_resolves_to_base(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            base = Path(tmpdir)
            result = resolve_safe_dir(".", base_dir=base)
            self.assertEqual(result, base.resolve())


if __name__ == "__main__":
    unittest.main()
