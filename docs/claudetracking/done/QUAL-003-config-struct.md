# QUAL-003: Introduce Config struct for run()

**Epic**: QUAL (Code Quality and Lint Fixes)
**Priority**: medium
**Depends on**: QUAL-002
**Status**: todo

## Goal

Replace the eight positional arguments to `run()` with a single `Config` struct. This resolves the `too_many_arguments` clippy warning and makes the API easier to extend as new flags are added.

## Success Criteria

- [ ] A public `Config` struct exists with all current parameters as fields.
- [ ] `run()` accepts `&Config` as its sole argument.
- [ ] `main.rs` constructs a `Config` from the parsed `Args`.
- [ ] The `#[allow(clippy::too_many_arguments)]` attribute (if added in QUAL-002) is removed.
- [ ] `cargo clippy --all-targets -- -D warnings` passes.
- [ ] All tests still pass and use the new API.
- [ ] `/update` has been run after changes.

## Steps

1. Define `pub struct Config` in `lib.rs` with fields: `base_path`, `matching`, `subfolders`, `prefix`, `suffix`, `recursive`, `dry_run`, `force`.
2. Update `run()` signature to `pub fn run(config: &Config) -> Result<()>`.
3. Update `main.rs` to build a `Config` from `Args`.
4. Update all test call sites.
5. Run fmt, clippy, and tests.

## Notes

Consider implementing `From<Args>` or a builder pattern if the struct grows further with v0.2.0 features.
