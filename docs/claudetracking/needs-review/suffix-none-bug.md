# Needs Review: --suffix none with multiple subfolders

## Finding

When `--suffix none` is used with `--subfolders > 1`, `format_folder_name()` returns the same folder name for every bucket (just the prefix with no distinguishing suffix). All files end up in a single folder.

This is arguably a bug. The tool silently does the wrong thing instead of warning the user.

## Options

1. **Error**: reject `--suffix none` when `--subfolders > 1`. Simple and safe.
2. **Warning**: allow it but print a warning that all files will be in one folder.
3. **Keep as-is**: document it as intentional (use `--suffix none` only with `--subfolders 1`).

## Recommendation

Option 1 (error) seems safest. It prevents user confusion with zero cost.

## Action needed

User decision on which option to implement.
