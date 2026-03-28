# TEST-003: Test error paths and edge cases

**Epic**: TEST (Expanded Test Coverage)
**Priority**: medium
**Depends on**: QUAL-001
**Status**: todo

## Goal

Add tests for error conditions: non-existent path, path is a file not a directory, zero subfolders, unknown suffix style, and the `--suffix none` edge case where all subfolders would have the same name.

## Success Criteria

- [ ] A test verifies that a non-existent path returns an error.
- [ ] A test verifies that a file path (not directory) returns an error.
- [ ] A test verifies that zero subfolders returns an error.
- [ ] A test verifies that an unknown suffix style returns an error.
- [ ] A test verifies that `--suffix none` with subfolders > 1 causes all files to land in one folder (documenting current behaviour).
- [ ] Tests pass.
- [ ] `/update` has been run after changes.

## Steps

1. Write each test case.
2. For the `--suffix none` case, document whether the current behaviour is intentional or a bug.

## Notes

The `--suffix none` with multiple subfolders is arguably a bug: `format_folder_name` returns the same name for every index, so all files go into one folder. This should probably be an error or a warning. File in needs-review.
