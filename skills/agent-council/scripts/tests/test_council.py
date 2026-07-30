#!/usr/bin/env python3
"""Unit tests for agent-council Python orchestrator (council.py)."""

import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import MagicMock, patch

SCRIPTS_DIR = Path(__file__).parent.parent.resolve()
sys.path.insert(0, str(SCRIPTS_DIR))

from council import (  # noqa: E402
    REPO_ROOT,
    clean_job,
    create_job,
    generate_job_id,
    get_results,
    load_config,
    parse_simple_yaml,
    update_job_status,
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

    def test_load_config_adr0005(self) -> None:
        config = load_config()
        self.assertIn("council", config)
        self.assertIn("members", config["council"])
        self.assertGreater(len(config["council"]["members"]), 0)

    def test_generate_job_id(self) -> None:
        job_id = generate_job_id("Test question")
        self.assertTrue(job_id.startswith("council-"))
        job_id2 = generate_job_id("Test question")
        self.assertEqual(job_id[-6:], job_id2[-6:])

    def test_create_and_clean_job(self) -> None:
        """Tests job lifecycle with subprocess.Popen mocked to avoid real CLI spawning."""
        with tempfile.TemporaryDirectory() as temp_dir:
            jobs_dir = Path(temp_dir)

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

            with patch("council.load_config", return_value=minimal_config), patch(
                "council.subprocess.Popen", return_value=mock_proc
            ) as mock_popen:
                job_dir = create_job("Unit test question", jobs_dir)

                mock_popen.assert_called_once()
                _, kwargs = mock_popen.call_args
                self.assertEqual(kwargs.get("cwd"), str(REPO_ROOT))

                self.assertTrue(job_dir.exists())
                self.assertTrue((job_dir / "job.json").exists())
                self.assertTrue((job_dir / "status.json").exists())

                with patch("council.os.kill", side_effect=OSError("no such process")):
                    status = update_job_status(job_dir)

                self.assertIn("overallState", status)
                self.assertEqual(status["overallState"], "done")

                res_text = get_results(job_dir, is_json=False)
                self.assertGreater(len(res_text), 0)

                clean_job(job_dir, jobs_base_dir=jobs_dir)
                self.assertFalse(job_dir.exists())

    def test_create_job_missing_cli(self) -> None:
        """Tests that create_job handles missing binary gracefully without raising FileNotFoundError."""
        with tempfile.TemporaryDirectory() as temp_dir:
            jobs_dir = Path(temp_dir)
            missing_config = {
                "council": {
                    "chairman": {"role": "auto"},
                    "members": [
                        {
                            "name": "missing-tool",
                            "command": "nonexistent_binary_xyz123 --flag",
                            "emoji": "🚫",
                            "color": "RED",
                        },
                    ],
                    "settings": {"exclude_chairman_from_members": True, "timeout": 60},
                }
            }

            with patch("council.load_config", return_value=missing_config):
                job_dir = create_job("Question for missing CLI", jobs_dir)
                self.assertTrue(job_dir.exists())

                status_file = job_dir / "status.json"
                self.assertTrue(status_file.exists())

                import json

                status = json.loads(status_file.read_text(encoding="utf-8"))
                self.assertIn("members", status)
                self.assertIn("missing-tool", status["members"])

                member_info = status["members"]["missing-tool"]
                self.assertEqual(member_info["state"], "missing_cli")
                self.assertIn("CLI executable 'nonexistent_binary_xyz123' not found in PATH", member_info["error"])

                err_file = job_dir / "missing-tool.err"
                self.assertTrue(err_file.exists())
                err_content = err_file.read_text(encoding="utf-8")
                self.assertIn("nonexistent_binary_xyz123", err_content)

                clean_job(job_dir, jobs_base_dir=jobs_dir)


if __name__ == "__main__":
    unittest.main()
