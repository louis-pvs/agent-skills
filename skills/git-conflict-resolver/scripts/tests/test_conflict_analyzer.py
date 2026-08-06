#!/usr/bin/env python3
"""Unit tests for Git Conflict Analyzer CLI Utility.

Verifies state detection, marker parsing (2-way and 3-way diffs), zero-marker verification,
and repository conflict analysis. Adheres to Strict Element-Level Test Assertion Rule.
"""

import sys
import tempfile
import unittest
from pathlib import Path

# Dynamic sys.path resolution (Repo Rule)
_repo_root = Path(__file__).resolve().parents[4]
if str(_repo_root) not in sys.path:
    sys.path.insert(0, str(_repo_root))

_script_dir = Path(__file__).resolve().parents[1]
if str(_script_dir) not in sys.path:
    sys.path.insert(0, str(_script_dir))

from conflict_analyzer import (  # noqa: E402
    analyze_repository_conflicts,
    detect_git_state,
    parse_conflict_markers,
    verify_zero_markers_in_content,
)


class TestConflictAnalyzer(unittest.TestCase):
    def test_parse_conflict_markers_two_way(self) -> None:
        content = """line 1
<<<<<<< HEAD
def foo():
    return "ours"
=======
def foo():
    return "theirs"
>>>>>>> feature-branch
line 8"""

        blocks = parse_conflict_markers(content)
        self.assertEqual(len(blocks), 1)

        block = blocks[0]
        self.assertIsInstance(block, dict)
        self.assertEqual(block["start_line"], 2)
        self.assertEqual(block["end_line"], 8)
        self.assertEqual(block["ours_label"], "HEAD")
        self.assertEqual(block["theirs_label"], "feature-branch")
        self.assertIsNone(block["base_content"])
        self.assertEqual(block["ours_content"], 'def foo():\n    return "ours"')
        self.assertEqual(block["theirs_content"], 'def foo():\n    return "theirs"')

    def test_parse_conflict_markers_three_way(self) -> None:
        content = """<<<<<<< HEAD
import os
import sys
||||||| 7a8b9c
import os
=======
import os
import sys
import json
>>>>>>> feature-branch"""

        blocks = parse_conflict_markers(content)
        self.assertEqual(len(blocks), 1)

        block = blocks[0]
        self.assertIsInstance(block, dict)
        self.assertEqual(block["start_line"], 1)
        self.assertEqual(block["end_line"], 10)
        self.assertEqual(block["ours_label"], "HEAD")
        self.assertEqual(block["base_label"], "7a8b9c")
        self.assertEqual(block["theirs_label"], "feature-branch")
        self.assertEqual(block["ours_content"], "import os\nimport sys")
        self.assertEqual(block["base_content"], "import os")
        self.assertEqual(block["theirs_content"], "import os\nimport sys\nimport json")

    def test_verify_zero_markers_clean_and_dirty(self) -> None:
        clean_content = "def hello():\n    return 'world'\n"
        is_clean, violations = verify_zero_markers_in_content(clean_content)
        self.assertTrue(is_clean)
        self.assertEqual(len(violations), 0)

        dirty_content = "def hello():\n<<<<<<< HEAD\n    return 'ours'\n=======\n    return 'theirs'\n>>>>>>> branch\n"
        is_clean_dirty, dirty_violations = verify_zero_markers_in_content(dirty_content)
        self.assertFalse(is_clean_dirty)
        self.assertEqual(len(dirty_violations), 3)
        self.assertEqual(dirty_violations[0], "Line 2: Unresolved start marker '<<<<<<<'")
        self.assertEqual(dirty_violations[1], "Line 4: Unresolved separator marker '======='")
        self.assertEqual(dirty_violations[2], "Line 6: Unresolved end marker '>>>>>>>'")

    def test_detect_git_state_temporary_dir(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            repo = Path(tmp_dir)
            git_dir = repo / ".git"
            git_dir.mkdir()

            clean_state = detect_git_state(repo)
            self.assertIsInstance(clean_state, dict)
            self.assertFalse(clean_state["is_in_conflict"])
            self.assertEqual(clean_state["operation"], "none")
            self.assertEqual(clean_state["unmerged_files_count"], 0)

            # Simulate merge conflict state
            (git_dir / "MERGE_HEAD").write_text("1234567890abcdef\n", encoding="utf-8")
            merge_state = detect_git_state(repo)
            self.assertTrue(merge_state["is_in_conflict"])
            self.assertEqual(merge_state["operation"], "merge")
            self.assertEqual(merge_state["merge_head"], "1234567890abcdef")

    def test_analyze_repository_conflicts_structure(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            repo = Path(tmp_dir)
            git_dir = repo / ".git"
            git_dir.mkdir()
            (git_dir / "MERGE_HEAD").write_text("123456\n", encoding="utf-8")

            conflicting_file = repo / "main.py"
            conflicting_file.write_text(
                "<<<<<<< HEAD\nprint('ours')\n=======\nprint('theirs')\n>>>>>>> main\n",
                encoding="utf-8",
            )

            report = analyze_repository_conflicts(repo)
            self.assertIsInstance(report, dict)
            self.assertIn("git_state", report)
            self.assertIn("commit_history", report)
            self.assertIn("analyzed_files", report)

            git_state = report["git_state"]
            self.assertEqual(git_state["operation"], "merge")
            self.assertTrue(git_state["is_in_conflict"])


if __name__ == "__main__":
    unittest.main()
