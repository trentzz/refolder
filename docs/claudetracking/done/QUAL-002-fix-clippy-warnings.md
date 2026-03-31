# QUAL-002: Fix clippy warnings

**Epic**: QUAL (Code Quality and Lint Fixes)
**Priority**: high
**Depends on**: QUAL-001
**Status**: done

## Goal

Make `cargo clippy --all-targets -- -D warnings` pass with zero errors. There are currently four clippy violations.

## Success Criteria

- [x] `cargo clippy --all-targets -- -D warnings` exits with code 0.
- [x] Empty line after doc comment on line 7-8 of `lib.rs` is fixed.
- [x] Needless range loop in `partition()` is replaced with iterator-based approach.
- [x] `vec![]` in test `integration_move_files` is replaced with an array literal.
- [x] All tests still pass.
- [ ] `/update` has been run after changes.

## Steps

1. Remove empty line between the module doc comment and `BOLD_START` constant (or move the doc comment to the correct item).
2. Refactor the `for i in 0..n` loop in `partition()` to use `enumerate()`.
3. Replace `vec![...]` with `[...]` in the test assertion.
4. Do NOT fix the `too_many_arguments` warning here; that is QUAL-003.
5. Temporarily allow `too_many_arguments` with a `#[allow()]` attribute until QUAL-003 is done.
6. Run clippy and tests.

## Notes

The `too_many_arguments` warning on `run()` requires a structural change (Config struct) and is handled separately in QUAL-003.
