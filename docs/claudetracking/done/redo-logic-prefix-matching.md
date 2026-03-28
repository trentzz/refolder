# Needs Review: Redo logic prefix matching is too broad

## Finding

The redo logic in `collect_files()` (line 183) uses `s.starts_with(prefix)` to identify existing target folders. This is overly broad. If the prefix is `"g"`, it will match `"git"`, `"generated"`, or any folder starting with `"g"`.

A more precise check would verify the folder name matches the exact pattern `prefix-<suffix>` (e.g. `group-1`, `group-a`).

## Options

1. **Regex match**: check that the folder name matches `^{prefix}-(\d+|[a-z]+)$`.
2. **Exact enumeration**: generate all expected folder names and check membership.
3. **Keep as-is**: document the behaviour and advise users to use distinctive prefixes.

## Recommendation

Option 2 (exact enumeration) is the most reliable. Generate the expected folder names based on the suffix style and subfolder count, then only collect from those folders.

## Action needed

User decision on whether this is worth fixing now or documenting as a known limitation.
