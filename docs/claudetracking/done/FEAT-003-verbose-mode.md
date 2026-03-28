# FEAT-003: --verbose mode

**Epic**: FEAT (v0.2.0 Feature Additions)
**Priority**: medium
**Depends on**: QUAL-003
**Status**: todo

## Goal

Add a `--verbose` (`-v`) flag that prints each file move as it happens. Useful for debugging and for users who want to see exactly what the tool does.

## Success Criteria

- [ ] A `--verbose` / `-v` flag is available on the CLI.
- [ ] When enabled, each move prints a line like: `moved: src -> dest`.
- [ ] Verbose output goes to stderr so it does not interfere with piped stdout.
- [ ] Dry-run output is unaffected by verbose mode.
- [ ] Tests verify verbose output.
- [ ] `/update` has been run after changes.

## Steps

1. Add `--verbose` flag to Args and Config.
2. In the move loop, print each move to stderr when verbose is enabled.
3. Add a test.

## Notes

Keep the format simple and machine-parseable. One line per move.
