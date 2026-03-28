# Epic: QUAL — Code Quality and Lint Fixes

## Goal

Bring the codebase to a clean state where `cargo fmt`, `cargo clippy`, and all tests pass without warnings. Refactor the public API to use a configuration struct instead of eight positional arguments.

## Motivation

The project currently fails both `cargo fmt --check` and `cargo clippy -- -D warnings`. The `main.rs` file has broken indentation (no indentation at all). The `run()` function takes eight arguments, which clippy flags as too many. These are basic hygiene issues that block any further development.

## Scope

- Fix formatting in `main.rs`.
- Fix all clippy warnings in `lib.rs` (empty doc comment line, too many arguments, needless range loop, useless vec in tests).
- Introduce a `Config` struct to replace the eight-argument `run()` signature.
- Replace `O(n)` linear search in `collect_files` deduplication with a `HashSet`.

## Child Tasks

- QUAL-001: Fix cargo fmt failures
- QUAL-002: Fix clippy warnings
- QUAL-003: Introduce Config struct for run()
- QUAL-004: Use HashSet for file deduplication in collect_files
