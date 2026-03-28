# TEST-001: Test --force flag behaviour

**Epic**: TEST (Expanded Test Coverage)
**Priority**: medium
**Depends on**: QUAL-001
**Status**: todo

## Goal

Add tests covering the `--force` flag: overwriting existing files in destination folders, and verifying that without `--force` an error is returned when a destination file exists.

## Success Criteria

- [ ] A test verifies that `--force` overwrites an existing destination file.
- [ ] A test verifies that without `--force`, an existing destination file causes an error.
- [ ] Tests pass.
- [ ] `/update` has been run after changes.

## Steps

1. Create a test that sets up a destination folder with a conflicting file.
2. Run without force and assert error.
3. Run with force and assert success.

## Notes

None.
