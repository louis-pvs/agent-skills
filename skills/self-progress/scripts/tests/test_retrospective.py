#!/usr/bin/env python3
"""Unit tests for self-progress retrospective script (TDD GREEN state)."""

import json
import sys
import tempfile
import unittest
from pathlib import Path
from typing import Any, Dict, List

# Dynamic sys.path resolution (Repo Rule)
repo_root = Path(__file__).resolve().parents[4]
if str(repo_root) not in sys.path:
    sys.path.insert(0, str(repo_root))

script_dir = Path(__file__).resolve().parents[1]
if str(script_dir) not in sys.path:
    sys.path.insert(0, str(script_dir))

from retrospective import extract_signals, parse_transcript  # noqa: E402


class TestRetrospective(unittest.TestCase):
    """Test suite for transcript log signal extraction."""

    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.transcript_path = Path(self.temp_dir.name) / "transcript.jsonl"

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def _create_sample_transcript(self, records: List[Dict[str, Any]]) -> Path:
        with open(self.transcript_path, "w", encoding="utf-8") as f:
            for record in records:
                f.write(json.dumps(record) + "\n")
        return self.transcript_path

    def test_extract_failed_tool_calls(self) -> None:
        records = [
            {
                "step_index": 1,
                "type": "PLANNER_RESPONSE",
                "status": "ERROR",
                "tool_calls": [{"name": "run_command", "args": {"CommandLine": "invalid_cmd"}}],
                "content": "Command failed exit 127",
            }
        ]
        self._create_sample_transcript(records)
        parsed = parse_transcript(self.transcript_path)
        signals = extract_signals(parsed)

        self.assertGreater(len(signals["errors"]), 0)
        self.assertEqual(signals["errors"][0]["tool_name"], "run_command")

    def test_extract_user_corrections(self) -> None:
        records = [
            {
                "step_index": 1,
                "type": "PLANNER_RESPONSE",
                "status": "ERROR",
                "tool_calls": [],
                "content": "Doing X...",
            },
            {
                "step_index": 2,
                "type": "USER_INPUT",
                "status": "DONE",
                "content": "No, don't do X, use Y pattern instead",
            },
        ]
        self._create_sample_transcript(records)
        parsed = parse_transcript(self.transcript_path)
        signals = extract_signals(parsed)

        self.assertGreater(len(signals["user_corrections"]), 0)
        self.assertIn("use Y pattern instead", signals["user_corrections"][0]["content"])

    def test_extract_research_patterns(self) -> None:
        records = [
            {
                "step_index": 1,
                "type": "PLANNER_RESPONSE",
                "status": "DONE",
                "tool_calls": [{"name": "search_web", "args": {"query": "how to build X in python"}}],
                "content": "Searching for missing knowledge",
            }
        ]
        self._create_sample_transcript(records)
        parsed = parse_transcript(self.transcript_path)
        signals = extract_signals(parsed)

        self.assertGreater(len(signals["research_patterns"]), 0)
        self.assertEqual(signals["research_patterns"][0]["query"], "how to build X in python")


if __name__ == "__main__":
    unittest.main()
