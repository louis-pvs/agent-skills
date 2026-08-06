# Git Conflict Resolution Strategies & Heuristics

Comprehensive reference rules and heuristics for resolving common code and configuration conflicts during merge and rebase.

## 1. Import Statement Conflicts

**Scenario**: Both branches added new imports at the top of the file.

**Resolution Strategy**:

1. **Combine All Unique Imports**: Preserve new imports added by both `OURS` and `THEIRS`.
2. **Remove Duplicates**: Eliminate duplicate import statements.
3. **Sort & Group**: Follow repository import ordering rules (e.g. stdlib first, third-party libraries second, local imports third).
4. **Remove Unused Imports**: Verify post-resolution that all imports are actually consumed in the file.

---

## 2. Function & Class Signature Conflicts

**Scenario**: Branch A updated a function signature (e.g. added a parameter or type annotation), while Branch B modified the function body or added new invocation sites.

**Resolution Strategy**:

1. **Preserve Updated Signature**: Keep parameter additions, type annotations, or async conversions from Branch A.
2. **Adapt Modifications**: Apply Branch B's body modifications to the updated signature.
3. **Update Call Sites**: Update any new call sites introduced by Branch B to include necessary arguments required by Branch A's signature.

---

## 3. Configuration & Dependency Conflicts (YAML, JSON, TOML)

**Scenario**: Both branches added new keys, dependencies, or configuration sections to files like `package.json`, `pyproject.toml`, or `config.yaml`.

**Resolution Strategy**:

1. **Structural Merge**: Merge entries at key/dictionary level rather than line level.
2. **Dependency Versioning**: If both branches added or bumped the same dependency version, select the higher compatible version constraint.
3. **Validate Syntax**: Parse the merged output with standard syntax parsers (`json.loads`, `yaml.safe_load`, etc.) to guarantee validity.

---

## 4. File Deletion vs File Modification (UD / DU / DD)

**Scenario**: One branch deleted a file while the other branch modified it.

**Resolution Strategy**:

1. **Inspect Commit History**: Determine why the file was deleted. Was it refactored/moved to a new path, or obsolete?
2. **If Refactored / Moved**: Apply the modifications from the other branch to the new file location.
3. **If Truly Obsolete**: Confirm deletion after ensuring no dependent code references the deleted symbols.
4. **Ask User when Ambiguous**: If intent cannot be inferred automatically, highlight the conflict and request user guidance.
