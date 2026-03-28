# INFRA-002: Prepare 0.2.0 release metadata

**Epic**: INFRA (CI and Tooling)
**Priority**: low
**Depends on**: FEAT-001, FEAT-002, FEAT-003
**Status**: todo

## Goal

Bump version to 0.2.0, update README with new flags, and ensure Cargo.toml metadata is complete for crates.io publishing.

## Success Criteria

- [ ] `Cargo.toml` version is `0.2.0`.
- [ ] README documents all new flags added in v0.2.0.
- [ ] `cargo publish --dry-run` succeeds.
- [ ] `/update` has been run after changes.

## Steps

1. Update version in `Cargo.toml`.
2. Update README with new flags and examples.
3. Run `cargo publish --dry-run` to verify.

## Notes

Do this task last, after all features are merged.
