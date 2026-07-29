# Architecture Audit Report Schema

Standard schema, calibration guidelines, and report formatting template for architecture audits.

---

## 1. Severity Levels

| Severity Level  | Definition                                                                                                                  | Example                                                                                                                  |
| --------------- | --------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| 🚨 **CRITICAL** | Severe architectural violation causing direct maintainability, safety, or scalability risk. Requires immediate refactoring. | Tightly-coupled DB driver calls scattered inside domain logic (violates DIP); God Object with 1000+ lines.               |
| ⚠️ **WARNING**  | Notable design flaw or principle violation that degrades codebase quality over time.                                        | Cascading switch statement across 10 types (violates OCP); 3rd instance of verbatim business calculation (violates DRY). |
| 💡 **ADVISORY** | Minor stylistic design suggestion or simplification opportunity.                                                            | Unclear domain variable naming; minor lambda simplification opportunity (violates CUPID/KISS).                           |

---

## 2. Confidence Ratings

| Rating        | Criteria                                                                                                               |
| ------------- | ---------------------------------------------------------------------------------------------------------------------- |
| **CONFIRMED** | Unambiguous violation backed by empirical code evidence (e.g. `NotImplementedError` in subclass, verbatim copy-paste). |
| **PLAUSIBLE** | Likely architectural smell based on heuristics, but subject to domain context or intentional design trade-off.         |

---

## 3. Standard Audit Report Template

````markdown
# 🏛️ Architecture Audit Report

**Target Scope**: `<target_path_or_diff>`
**Principles Evaluated**: `SOLID, DRY, YAGNI, KISS, CUPID`
**Overall Code Quality Rating**: `[A / B / C / D / F]`

---

## Summary of Findings

- 🚨 **Critical**: X
- ⚠️ **Warning**: Y
- 💡 **Advisory**: Z

---

## Detailed Findings & Refactoring Recommendations

### 1. [Severity Emoji] [Principle Name] Violation: [Short Summary]

- **File**: `[path/to/file.py:L10-L45](file:///absolute/path/to/file.py#L10-L45)`
- **Severity**: `CRITICAL` | `WARNING` | `ADVISORY`
- **Confidence**: `CONFIRMED` | `PLAUSIBLE`
- **Impact**: Detailed explanation of why this hurts maintainability, testability, or extensibility.

#### Code Snippet (Current)

```python
# Current code snippet
```
````

#### Proposed Refactoring (Suggested Diff)

```python
# Refactored code snippet
```

---

## Principle Trade-off Analysis

_Highlights any design tensions evaluated (e.g., why DRY was prioritized over YAGNI in this instance)._

```

```
