#!/usr/bin/env python3
"""Drift guard for the duplicated resolve_project_dir helper.

Each script under scripts/ is intentionally standalone (ADR 0001 - no cross-script
imports), so resolve_project_dir is duplicated per file. These tests pin the shared
behavioural contract so the copies cannot silently diverge again.
"""

import importlib
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPTS_DIR = Path(__file__).parent.parent.resolve()
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

_MODULES_WITH_HELPER = ("impact_analyzer", "scenario_runner", "failure_preemptor")


class TestResolveProjectDirContract(unittest.TestCase):
    def _helpers(self):
        for name in _MODULES_WITH_HELPER:
            module = importlib.import_module(name)
            yield name, module.resolve_project_dir

    def test_every_script_still_defines_the_helper(self) -> None:
        """All standalone copies must remain present and callable."""
        found = [name for name, fn in self._helpers() if callable(fn)]
        self.assertEqual(found, list(_MODULES_WITH_HELPER))

    def test_absolute_paths_are_preserved(self) -> None:
        """An absolute --dir must resolve to itself, never to a cwd-relative basename."""
        with tempfile.TemporaryDirectory() as temp_dir:
            expected = Path(temp_dir).resolve()
            for name, resolve in self._helpers():
                with self.subTest(module=name):
                    self.assertEqual(resolve(str(expected)), expected)

    def test_all_copies_agree_on_the_same_input(self) -> None:
        """Every copy must return an identical result for identical input."""
        with tempfile.TemporaryDirectory() as temp_dir:
            results = {name: resolve(temp_dir) for name, resolve in self._helpers()}
            self.assertEqual(len(set(results.values())), 1, f"copies diverged: {results}")

    def test_missing_directory_exits_nonzero(self) -> None:
        """A non-existent directory must abort rather than silently continue."""
        missing = str(Path(tempfile.gettempdir()) / "wia-does-not-exist-xyz")
        for name, resolve in self._helpers():
            with self.subTest(module=name):
                with self.assertRaises(SystemExit) as ctx:
                    resolve(missing)
                self.assertNotEqual(ctx.exception.code, 0)


if __name__ == "__main__":
    unittest.main()
