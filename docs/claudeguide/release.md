# Release Process

## Registry

crates.io

## Publish command

```
cargo publish
```

## Pre-publish checklist

- [ ] `cargo fmt -- --check` passes
- [ ] `cargo clippy --all-targets -- -D warnings` passes
- [ ] `cargo test` passes
- [ ] Version bumped in `Cargo.toml`
- [ ] `CHANGELOG.md` updated
- [ ] Release branch pushed to remote

## Auth

Run `cargo login` with a crates.io API token before publishing.

## What is included

Only source code and README. Cargo automatically excludes `.git/`, `target/`, and anything in `.gitignore`.

## Explicit excludes (add to Cargo.toml if needed)

- `docs/`
- `.github/`

## Post-publish

- Tag the commit: `git tag v<version>`
- Push the tag: `git push origin v<version>`
- Merge release branch to main
