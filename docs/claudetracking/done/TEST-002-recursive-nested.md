# TEST-002: Test --recursive with nested directories

**Epic**: TEST (Expanded Test Coverage)
**Priority**: medium
**Depends on**: QUAL-001
**Status**: todo

## Goal

Add tests covering `--recursive` with multi-level nested directories. The current tests do not exercise the recursive flag at all.

## Success Criteria

- [ ] A test creates a directory tree with files at multiple levels.
- [ ] Running with `--recursive` collects files from all levels.
- [ ] Running without `--recursive` only collects files from the top level.
- [ ] Tests pass.
- [ ] `/update` has been run after changes.

## Steps

1. Create a test with a nested directory structure (e.g. `a/b/c/file.txt`).
2. Run with `recursive=false` and verify only top-level files are collected.
3. Run with `recursive=true` and verify all files are collected.

## Notes

Be careful with the interaction between recursive mode and the redo logic (collecting from existing prefix folders). If `--recursive` is true and prefix folders contain nested directories, the behaviour may be surprising.
