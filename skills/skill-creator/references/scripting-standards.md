# Repository Scripting & Testing Standards

All scripts developed within skills in this repository must adhere to the following standards to ensure zero-prerequisite installation, cross-platform portability, high LLM readability, and automated testability.

---

## 1. Mandatory Language Standard: Python 3 (Standard Library First)

- **Primary Language**: Python 3.8+
- **Standard Library First Rule**: Scripts MUST rely on Python's standard library modules (`pathlib`, `json`, `argparse`, `subprocess`, `urllib`, `dataclasses`, `asyncio`, `unittest`) whenever possible.
- **Why**: Standard library scripts require zero `pip install` or package management prerequisites. They run out-of-the-box on Linux, macOS, Windows, and containerized agent environments.

---

## 2. Directory Layout & Organization

Every skill containing executable code must follow this structure:

```text
skills/<skill-name>/
└── scripts/
    ├── my_script.py         # Main executable script
    └── tests/
        └── test_my_script.py # Unit tests using unittest
```

---

## 3. Coding Guidelines & Standards

### Cross-Platform Path Handling

- **Never hardcode string paths with slashes**: Avoid `path = "dir/" + filename`.
- **Always use `pathlib.Path`**:

  ```python
  from pathlib import Path

  script_dir = Path(__file__).parent.resolve()
  target_file = script_dir / "data" / "output.json"
  ```

### CLI Interface Specification

- Use `argparse` with clear parameter descriptions, positional arguments, and `--help` flags:

  ```python
  import argparse


  def parse_args():
      parser = argparse.ArgumentParser(description="Skill script utility.")
      parser.add_argument("--input", required=True, help="Input file path")
      return parser.parse_args()
  ```

### Type Hinting & Docstrings

- Include explicit type hints from `typing` for all function signatures.
- Write concise docstrings for all top-level functions and classes.

---

## 4. Automated Testing Standard

### Test Framework

- Use Python's built-in `unittest` framework.
- Place test files in `scripts/tests/test_<script_name>.py`.

### Test Execution Command

Run the test suite using standard discovery:

```bash
python3 -m unittest discover -s skills/<skill-name>/scripts/tests
```

### Mocking System & Subprocess Calls

- Use `unittest.mock.patch` and `unittest.mock.MagicMock` to isolate file I/O and external process executions.

---

## 5. Shell Script Wrappers (Restricted Exception)

Shell scripts (`.sh`) are permitted **only as thin wrappers** under the following conditions:

1. Must be under **20 lines** of total code.
2. Must include strict bash flags at the top:

   ```bash
   #!/usr/bin/env bash
   set -euo pipefail
   ```

3. Must immediately delegate execution to Python or a binary tool.
