# Janitor Audit Report Schema

Template and schema for generating structured cleanup reports.

## Report Structure

```markdown
# Code Janitor Report

**Scan Target**: `<path>`
**Scan Date**: `<ISO-8601 timestamp>`
**Total Files Scanned**: `<count>`

## Summary

| Metric | Count |
| :--- | ---: |
| Total Smells Detected | X |
| 🚨 Critical | X |
| ⚠️ Warning | X |
| 💡 Advisory | X |
| Auto-Fixed | X |
| Manual Review Required | X |

## Findings

### File: `<relative/path/to/file.py>`

#### 🚨 CRITICAL: <Smell Name>

- **Line(s)**: L42-L45
- **Description**: Brief explanation of the issue.
- **Fix Applied**: Yes / No / Manual Review Required

\`\`\`diff
-<old code>
+<new code>
\`\`\`

#### ⚠️ WARNING: <Smell Name>

- **Line(s)**: L100-L145
- **Description**: Brief explanation.
- **Fix Applied**: No (MANUAL_REVIEW_REQUIRED)
- **Suggested Refactoring**: Extract method or apply guard clause.

### File: `<relative/path/to/another_file.py>`

...

## Hygiene Score

| Before | After | Improvement |
| :---: | :---: | :---: |
| X smells | Y smells | Z% reduction |

## Deferred Items

Items flagged as `MANUAL_REVIEW_REQUIRED` that need human attention:

- [ ] `file.py:L100` — Oversized function `process_all` (45 lines). Test coverage
  insufficient for safe automated extraction.
- [ ] `utils.py:L22` — TODO marker references external ticket #1234.
```

## Field Definitions

- **Smell Name**: One of the categories from the code smells catalog.
- **Severity**: `🚨 CRITICAL`, `⚠️ WARNING`, or `💡 ADVISORY`.
- **Fix Applied**: `Yes` (auto-applied), `No` (deferred), or `Manual Review Required`.
- **Hygiene Score**: Percentage reduction in total smell count after applied fixes.
