# FEAT-002: --no-color flag and TTY detection

**Epic**: FEAT (v0.2.0 Feature Additions)
**Priority**: high
**Depends on**: QUAL-003
**Status**: todo

## Goal

Add a `--no-color` flag and detect non-TTY output to disable ANSI escape codes. Currently the tool always emits ANSI bold codes in dry-run output, which produces garbage when piped or used in CI.

## Success Criteria

- [ ] A `--no-color` flag is available on the CLI.
- [ ] When stdout is not a TTY, ANSI codes are suppressed automatically.
- [ ] When `--no-color` is passed, ANSI codes are suppressed regardless of TTY.
- [ ] The `NO_COLOR` environment variable is respected (see https://no-color.org/).
- [ ] The hardcoded `BOLD_START`/`BOLD_END` constants are replaced with a runtime colour decision.
- [ ] Tests pass with and without colour.
- [ ] `/update` has been run after changes.

## Steps

1. Add `--no-color` flag to Args struct.
2. Check `std::io::stdout().is_terminal()` (available in std since Rust 1.70) and `NO_COLOR` env var.
3. Replace hardcoded ANSI constants with a function that returns empty strings when colour is disabled.
4. Pass the colour decision through to `print_dry_run_preview`.
5. Add tests.

## Notes

Consider using the `supports-color` crate or just the stdlib `IsTerminal` trait. The stdlib approach avoids a new dependency.
