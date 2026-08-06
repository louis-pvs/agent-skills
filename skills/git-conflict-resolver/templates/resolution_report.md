# Git Conflict Resolution Report

**Git Operation**: `{operation}` (`{ours_branch}` ↔ `{theirs_branch}`)
**Timestamp**: `{timestamp}`
**Resolved Files Count**: `{resolved_count}`

---

## Resolved Files

### `{file_path}`

- **Conflict Count**: `{block_count}` blocks
- **Resolution Strategy**: `{strategy_summary}`
- **Changes Made**:
  - Combined imports from both branches.
  - Adapted function signature to preserve new parameters while incorporating logic updates.
- **Verification Status**:
  - Zero conflict markers: ✅ PASS
  - Linter check: ✅ PASS
  - Tests check: ✅ PASS

---

## Verification & Next Steps

1. Run `python3 skills/git-conflict-resolver/scripts/conflict_analyzer.py --verify` to re-confirm zero conflict markers.
2. Stage resolved files: `git add <files>`
3. Continue Git operation: `git rebase --continue` or `git merge --continue`
