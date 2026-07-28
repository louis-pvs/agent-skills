import shutil
import tempfile
import unittest
from pathlib import Path

from scripts.install_global import (
    find_repo_skills,
    install_skill,
    uninstall_skill,
)


class TestInstallGlobal(unittest.TestCase):
    def setUp(self):
        self.temp_dir = Path(tempfile.mkdtemp())
        self.skills_dir = self.temp_dir / "skills"
        self.skills_dir.mkdir()

        # Create dummy skills
        self.skill_a = self.skills_dir / "skill-a"
        self.skill_a.mkdir()
        (self.skill_a / "SKILL.md").write_text("---\nname: skill-a\ndescription: test\n---", encoding="utf-8")

        self.skill_b = self.skills_dir / "skill-b"
        self.skill_b.mkdir()
        (self.skill_b / "SKILL.md").write_text("---\nname: skill-b\ndescription: test\n---", encoding="utf-8")

        self.target_dir = self.temp_dir / "global_skills"

    def tearDown(self):
        shutil.rmtree(self.temp_dir, ignore_errors=True)

    def test_find_repo_skills(self):
        skills = find_repo_skills(self.skills_dir)
        names = [s.name for s in skills]
        self.assertEqual(names, ["skill-a", "skill-b"])

    def test_install_and_uninstall_skill(self):
        # Install skill-a
        msg = install_skill(self.skill_a, self.target_dir)
        self.assertIn("[LINKED]", msg)

        target_link = self.target_dir / "skill-a"
        self.assertTrue(target_link.is_symlink())
        self.assertEqual(target_link.resolve(), self.skill_a.resolve())

        # Re-install should detect existing link
        msg_re = install_skill(self.skill_a, self.target_dir)
        self.assertIn("[EXISTS]", msg_re)

        # Uninstall skill-a
        msg_un = uninstall_skill(self.skill_a, self.target_dir)
        self.assertIn("[REMOVED]", msg_un)
        self.assertFalse(target_link.exists())

    def test_dry_run(self):
        msg = install_skill(self.skill_a, self.target_dir, dry_run=True)
        self.assertIn("[DRY-RUN]", msg)
        self.assertFalse((self.target_dir / "skill-a").exists())


if __name__ == "__main__":
    unittest.main()
