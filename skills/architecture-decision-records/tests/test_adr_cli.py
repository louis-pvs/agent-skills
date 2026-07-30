"""Unit tests for adr_cli module."""

import importlib.util
import unittest
from pathlib import Path

# Dynamic import of adr_cli module from folder with hyphens
script_path = Path(__file__).resolve().parents[1] / "scripts" / "adr_cli.py"
spec = importlib.util.spec_from_file_location("adr_cli", script_path)
adr_cli = importlib.util.module_from_spec(spec)
spec.loader.exec_module(adr_cli)


class TestAdrCli(unittest.TestCase):
    def test_slugify(self):
        """Test text slugification to kebab-case filenames."""
        self.assertEqual(adr_cli.slugify("Use FastAPI for REST Endpoints"), "use-fastapi-for-rest-endpoints")
        self.assertEqual(adr_cli.slugify("ADR 0001: Initial Setup!"), "adr-0001-initial-setup")
        self.assertEqual(adr_cli.slugify("  Spaces & Special---Chars  "), "spaces-special-chars")

    def test_init_adr_repo(self):
        """Test repo initialization with 0000 ADR and README index."""
        import tempfile

        with tempfile.TemporaryDirectory() as tmp_dir:
            target_dir = Path(tmp_dir) / "docs" / "adr"
            result_path = adr_cli.init_adr_repo(str(target_dir))

            self.assertIsInstance(result_path, Path)
            self.assertTrue(result_path.exists())

            adr_0000 = target_dir / "0000-use-markdown-architectural-decision-records.md"
            index_file = target_dir / "README.md"

            self.assertTrue(adr_0000.exists())
            self.assertTrue(index_file.exists())

            # Strict element-level assertion on content
            adr_content = adr_0000.read_text(encoding="utf-8")
            self.assertIn("* Status: Accepted", adr_content)
            self.assertIn("# 0. Use Markdown Architectural Decision Records", adr_content)

            index_content = index_file.read_text(encoding="utf-8")
            self.assertIn("| 0000 | [0. Use Markdown Architectural Decision Records]", index_content)
            self.assertIn("| Accepted |", index_content)

    def test_new_adr_generation(self):
        """Test sequential ADR file generation and ID incrementing."""
        import tempfile

        with tempfile.TemporaryDirectory() as tmp_dir:
            target_dir = Path(tmp_dir) / "docs" / "adr"
            adr_cli.init_adr_repo(str(target_dir))

            # Create ADR 0001
            file_1 = adr_cli.new_adr("Adopt PostgreSQL for Persistence", str(target_dir))
            self.assertIsInstance(file_1, Path)
            self.assertEqual(file_1.name, "0001-adopt-postgresql-for-persistence.md")

            meta_1 = adr_cli.parse_adr_metadata(file_1)
            self.assertEqual(meta_1["filename"], "0001-adopt-postgresql-for-persistence.md")
            self.assertEqual(meta_1["title"], "1. Adopt PostgreSQL for Persistence")
            self.assertEqual(meta_1["status"], "Proposed")
            self.assertGreater(len(meta_1["date"]), 0)

            # Create ADR 0002
            file_2 = adr_cli.new_adr("Use Redis for Session Caching", str(target_dir))
            self.assertIsInstance(file_2, Path)
            self.assertEqual(file_2.name, "0002-use-redis-for-session-caching.md")

            self.assertEqual(adr_cli.find_highest_adr_id(target_dir), 2)

            # Check file list
            all_files = adr_cli.get_all_adr_files(target_dir)
            self.assertEqual(len(all_files), 3)
            self.assertEqual(
                [f.name for f in all_files],
                [
                    "0000-use-markdown-architectural-decision-records.md",
                    "0001-adopt-postgresql-for-persistence.md",
                    "0002-use-redis-for-session-caching.md",
                ],
            )
            for item in all_files:
                self.assertIsInstance(item, Path)

    def test_supersede_adr_workflow(self):
        """Test superseding an old ADR with a new ADR."""
        import tempfile

        with tempfile.TemporaryDirectory() as tmp_dir:
            target_dir = Path(tmp_dir) / "docs" / "adr"
            adr_cli.init_adr_repo(str(target_dir))

            old_file = adr_cli.new_adr("Use MySQL Database", str(target_dir))
            new_file = adr_cli.new_adr("Migrate to PostgreSQL Database", str(target_dir))

            adr_cli.supersede_adr(1, 2, str(target_dir))

            # Strict content assertion on updated status line
            old_content = old_file.read_text(encoding="utf-8")
            self.assertIn(
                "* Status: Superseded by [0002-migrate-to-postgresql-database](0002-migrate-to-postgresql-database.md)",
                old_content,
            )

            new_content = new_file.read_text(encoding="utf-8")
            self.assertIn("* Status: Proposed", new_content)

            # Verify index updated
            index_content = (target_dir / "README.md").read_text(encoding="utf-8")
            self.assertIn("Superseded by [0002-migrate-to-postgresql-database]", index_content)

    def test_validate_adrs(self):
        """Test validation scanner for gaps or formatting errors."""
        import tempfile

        with tempfile.TemporaryDirectory() as tmp_dir:
            target_dir = Path(tmp_dir) / "docs" / "adr"
            adr_cli.init_adr_repo(str(target_dir))
            adr_cli.new_adr("First Real Decision", str(target_dir))

            res = adr_cli.validate_adrs(str(target_dir))
            self.assertIsInstance(res, dict)
            self.assertIsInstance(res["errors"], list)
            self.assertIsInstance(res["warnings"], list)
            self.assertEqual(len(res["errors"]), 0)

    def test_dry_run(self):
        """Test that --dry-run previews actions without mutating disk."""
        import tempfile

        with tempfile.TemporaryDirectory() as tmp_dir:
            target_dir = Path(tmp_dir) / "docs" / "adr"
            res = adr_cli.init_adr_repo(str(target_dir), dry_run=True)
            self.assertIsInstance(res, Path)
            self.assertFalse(target_dir.exists())


if __name__ == "__main__":
    unittest.main()
