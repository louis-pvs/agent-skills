import sys
import tempfile
import unittest
from pathlib import Path

# Add scripts directory to sys.path for test execution
SCRIPTS_DIR = Path(__file__).resolve().parents[1]
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

from doc_auditor import audit_markdown_file  # noqa: E402


class TestDocAuditor(unittest.TestCase):
    def test_valid_markdown_file(self):
        content = """# Valid Title

> [!NOTE]
> This is a valid alert.

## Architecture

```mermaid
graph TD
    A --> B
```

See [docs](docs/readme.md).
"""
        with tempfile.NamedTemporaryFile("w+", suffix=".md", delete=False) as tmp:
            tmp.write(content)
            tmp_path = Path(tmp.name)

        try:
            passed, findings = audit_markdown_file(tmp_path)
            self.assertTrue(passed)
            self.assertEqual(len([f for f in findings if f.startswith("ERROR")]), 0)
        finally:
            tmp_path.unlink()

    def test_multiple_h1_titles(self):
        content = """# First Title

# Second Title
"""
        with tempfile.NamedTemporaryFile("w+", suffix=".md", delete=False) as tmp:
            tmp.write(content)
            tmp_path = Path(tmp.name)

        try:
            passed, findings = audit_markdown_file(tmp_path)
            self.assertFalse(passed)
            self.assertTrue(any("Multiple H1 titles" in f for f in findings))
        finally:
            tmp_path.unlink()

    def test_invalid_gfm_alert(self):
        content = """# Document Title

> [!UNKNOWN]
> Invalid alert tag.
"""
        with tempfile.NamedTemporaryFile("w+", suffix=".md", delete=False) as tmp:
            tmp.write(content)
            tmp_path = Path(tmp.name)

        try:
            passed, findings = audit_markdown_file(tmp_path)
            self.assertFalse(passed)
            self.assertTrue(any("invalid GFM alert tag" in f for f in findings))
        finally:
            tmp_path.unlink()

    def test_privacy_user_path(self):
        content = """# Document Title

Path: /home/john_doe/secret_project/file.txt
"""
        with tempfile.NamedTemporaryFile("w+", suffix=".md", delete=False) as tmp:
            tmp.write(content)
            tmp_path = Path(tmp.name)

        try:
            passed, findings = audit_markdown_file(tmp_path)
            self.assertFalse(passed)
            self.assertTrue(any("hardcoded personal user path" in f for f in findings))
        finally:
            tmp_path.unlink()


if __name__ == "__main__":
    unittest.main()
