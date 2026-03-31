# FEAT-005: --output-dir flag

**Epic**: FEAT (v0.2.0 Feature Additions)
**Priority**: medium
**Depends on**: QUAL-003
**Status**: todo

## Goal

Add an `--output-dir` flag that specifies where the subfolders are created. Currently subfolders are always created inside the source directory. Users may want to distribute files into a different location.

## Success Criteria

- [ ] An `--output-dir` / `-o` flag is available on the CLI.
- [ ] When specified, subfolders are created under the output directory instead of the source directory.
- [ ] When not specified, behaviour is unchanged (subfolders in source directory).
- [ ] The output directory is created if it does not exist.
- [ ] Dry-run preview shows the correct output paths.
- [ ] Tests cover the `--output-dir` case.
- [ ] `/update` has been run after changes.

## Steps

1. Add `--output-dir` flag to Args and Config.
2. In `run()`, use the output directory as the base for subfolder creation when specified.
3. Update dry-run preview to reflect output paths.
4. Add tests.

## Notes

When `--output-dir` is used, the "redo" logic (collecting files from existing prefix folders) should look in both the source and output directories.
