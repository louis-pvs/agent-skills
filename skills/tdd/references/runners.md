# Supported Test Runners & Detection Matrix

The `tdd_runner.py` tool automatically inspects the target directory for framework configuration files and test patterns.

| Language / Environment | Trigger Indicators | Auto-Detected Command |
| :--- | :--- | :--- |
| **Python (pytest)** | `pyproject.toml`, `pytest.ini`, `setup.cfg` | `pytest` |
| **Python (unittest)** | `test_*.py`, `*_test.py` files | `python3 -m unittest discover -s .` |
| **Node.js (npm)** | `package.json` with `"test"` script | `npm test` |
| **Node.js (Jest)** | `package.json` (fallback) | `npx jest` |
| **Go** | `go.mod` | `go test ./...` |
| **Rust** | `Cargo.toml` | `cargo test` |

## Overriding Detection

If a project uses custom test paths, non-standard arguments, or flags, specify `--cmd`:

```bash
python3 skills/tdd/scripts/tdd_runner.py --cmd "pytest tests/unit/test_api.py -v" --verify-green
```
