# QUAL-001: Fix cargo fmt failures

**Epic**: QUAL (Code Quality and Lint Fixes)
**Priority**: high
**Depends on**: none
**Status**: done

## Goal

Make `cargo fmt -- --check` pass. The `main.rs` file has no indentation and extra blank lines throughout. This is the most basic prerequisite for any other work.

## Success Criteria

- [x] `cargo fmt -- --check` exits with code 0.
- [x] No manual formatting overrides or `#[rustfmt::skip]` attributes added.
- [x] All tests still pass.
- [ ] `/update` has been run after changes.

## Steps

1. Run `cargo fmt` to auto-fix formatting.
2. Verify with `cargo fmt -- --check`.
3. Run `cargo test` to confirm nothing broke.

## Notes

The entire `main.rs` file appears to have been written without indentation. Running `cargo fmt` should fix it completely.
