# FEAT-006: Shell completions generation

**Epic**: FEAT (v0.2.0 Feature Additions)
**Priority**: low
**Depends on**: QUAL-003
**Status**: todo

## Goal

Add a `--completions <SHELL>` flag that generates shell completion scripts for bash, zsh, fish, and PowerShell using clap's built-in completion generation.

## Success Criteria

- [ ] A `--completions` flag accepts shell names: bash, zsh, fish, powershell.
- [ ] Running `refolder --completions bash` prints a valid bash completion script to stdout.
- [ ] The flag is documented in the README.
- [ ] `/update` has been run after changes.

## Steps

1. Add `clap_complete` to dependencies.
2. Add a `--completions` flag to Args.
3. When the flag is present, generate completions and exit.
4. Update README.

## Notes

This is low priority but trivial to implement with clap_complete.
