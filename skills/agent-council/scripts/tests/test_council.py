#!/usr/bin/env python3
"""Unit tests for agent-council Python orchestrator (council.py)."""

import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch, MagicMock
import sys

SCRIPTS_DIR = Path(__file__).parent.parent.resolve()
sys.path.insert(0, str(SCRIPTS_DIR))

from council import (
    parse_simple_yaml,
    generate_job_id,
    create_job,
    update_job_status,
    get_results,
    clean_job,
)


class TestCouncilOrchestrator(unittest.TestCase):

    def test_parse_simple_yaml(self) -> None:
        sample_yaml = """
council:
  settings:
    timeout: 120
    exclude_chairman: true
"""
        parsed = parse_simple_yaml(sample_yaml)
        self.assertIn("council", parsed)

    def test_generate_job_id(self) -> None:
        job_id = generate_job_id("Test question")
        self.assertTrue(job_id.startswith("council-"))
        # Second call with same question should produce same hash suffix
        job_id2 = generate_job_id("Test question")
        self.assertEqual(job_id[-6:], job_id2[-6:])

    def test_create_and_clean_job(self) -> None:
        """Tests job lifecycle with subprocess.Popen mocked to avoid real CLI spawning."""
        with tempfile.TemporaryDirectory() as temp_dir:
            jobs_dir = Path(temp_dir)

            # Mock Popen so no real subprocess is spawned during tests
            mock_proc = MagicMock()
            mock_proc.pid = 99999

            minimal_config = {
                "council": {
                    "chairman": {"role": "auto"},
                    "members": [
                        {"name": "mock-agent", "command": "echo", "emoji": "🤖", "color": "GREEN"},
                    ],
                    "settings": {"exclude_chairman_from_members": True, "timeout": 60},
                }
            }

            with patch("council.load_config", return_value=minimal_config), \
                 patch("council.subprocess.Popen", return_value=mock_proc):

                job_dir = create_job("Unit test question", jobs_dir)

                self.assertTrue(job_dir.exists())
                self.assertTrue((job_dir / "job.json").exists())
                self.assertTrue((job_dir / "status.json").exists())

                # Update status (mock process is not running, so it becomes 'done')
                with patch("council.os.kill", side_effect=OSError("no such process")):
                    status = update_job_status(job_dir)

                self.assertIn("overallState", status)
                self.assertEqual(status["overallState"], "done")

                # Get results should return non-empty text
                res_text = get_results(job_dir, is_json=False)
                self.assertGreater(len(res_text), 0)

                # Cleanup
                clean_job(job_dir)
                self.assertFalse(job_dir.exists())


if __name__ == "__main__":
    unittest.main()
