# INFRA-001: Add GitHub Actions CI workflow

**Epic**: INFRA (CI and Tooling)
**Priority**: medium
**Depends on**: QUAL-002
**Status**: todo

## Goal

Add a GitHub Actions workflow that runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `cargo audit` on every push and pull request.

## Success Criteria

- [ ] A `.github/workflows/ci.yml` file exists.
- [ ] The workflow runs on push to `main` and on pull requests.
- [ ] The workflow checks formatting, clippy, tests, and audit.
- [ ] The workflow uses a recent stable Rust toolchain.
- [ ] `/update` has been run after changes.

## Steps

1. Create `.github/workflows/ci.yml`.
2. Define jobs for fmt, clippy, test, and audit.
3. Use `actions-rs` or direct `rustup`/`cargo` commands.

## Notes

Keep it simple. A single job with sequential steps is fine for a small project.
