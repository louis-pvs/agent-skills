#!/usr/bin/env python3
"""Unit tests for Phase 2 scenario runner and fallback tradeoff evaluator (scenario_runner.py)."""

import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

SCRIPTS_DIR = Path(__file__).parent.parent.resolve()
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

from scenario_runner import (  # noqa: E402
    check_council_availability,
    evaluate_single_agent_scenarios,
    generate_scenario_tradeoff_matrix,
    run_scenario_probe,
)


class TestScenarioRunner(unittest.TestCase):
    def test_check_council_availability(self) -> None:
        """Verify detection of council.py script availability."""
        with tempfile.TemporaryDirectory() as temp_dir:
            fake_root = Path(temp_dir)
            # When file doesn't exist
            self.assertFalse(check_council_availability(fake_root))

            # Create mock council.py script
            council_script = fake_root / "skills" / "agent-council" / "scripts" / "council.py"
            council_script.parent.mkdir(parents=True, exist_ok=True)
            council_script.write_text("#!/usr/bin/env python3\nprint('mock council')", encoding="utf-8")

            self.assertTrue(check_council_availability(fake_root))

    def test_evaluate_single_agent_scenarios(self) -> None:
        """Verify fallback evaluation returns honest LOW confidence for heuristic guesses."""
        scenarios = ["Option A: Refactor to Async", "Option B: Retain Subprocess", "Option C: Unmatched Scenery"]
        results = evaluate_single_agent_scenarios("parse_simple_yaml", scenarios)

        self.assertIn("parse_simple_yaml", results["symbol"])
        self.assertEqual(results["mode"], "single_agent_fallback")
        self.assertEqual(len(results["scenarios"]), 3)

        # Heuristic guesses MUST be labeled LOW confidence
        self.assertEqual(results["scenarios"][0]["confidence"], "LOW")
        self.assertEqual(results["scenarios"][1]["confidence"], "LOW")

        # Unmatched scenario must have Unknown latency & LOW confidence
        self.assertIn("Unknown", results["scenarios"][2]["latency_impact"])
        self.assertEqual(results["scenarios"][2]["confidence"], "LOW")

    def test_generate_scenario_tradeoff_matrix(self) -> None:
        """Verify GFM tradeoff matrix report formatting."""
        mock_data = {
            "symbol": "target_symbol",
            "mode": "single_agent_fallback",
            "scenarios": [
                {
                    "name": "Option A: Async Dispatch",
                    "latency_impact": "Fast (-80ms)",
                    "blast_radius_risk": "MEDIUM",
                    "maintenance_cost": "Low",
                    "confidence": "LOW",
                },
                {
                    "name": "Option B: Subprocess Fork",
                    "latency_impact": "Base (+250ms)",
                    "blast_radius_risk": "LOW",
                    "maintenance_cost": "Low",
                    "confidence": "LOW",
                },
            ],
        }

        report = generate_scenario_tradeoff_matrix(mock_data)
        self.assertIn("Scenario Tradeoff Matrix", report)
        self.assertIn("Option A: Async Dispatch", report)
        self.assertIn("Option B: Subprocess Fork", report)
        self.assertIn("Single-Agent Analytical Fallback", report)

    def test_run_scenario_probe_fallback_branch(self) -> None:
        """Verify scenario probe executes fallback when council is unavailable."""
        scenarios = ["Option 1", "Option 2"]
        with patch("scenario_runner.check_council_availability", return_value=False):
            results = run_scenario_probe("test_symbol", scenarios, Path("."))
            self.assertEqual(results["mode"], "single_agent_fallback")


if __name__ == "__main__":
    unittest.main()
