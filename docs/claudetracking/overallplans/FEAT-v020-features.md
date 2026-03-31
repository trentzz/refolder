# Epic: FEAT — v0.2.0 Feature Additions

## Goal

Add features that make refolder substantially more useful for its target audience. Focus on output visibility, CI compatibility, and common workflow needs.

## Motivation

The current tool silently succeeds on non-dry-run operations, giving the user no confirmation. It always emits ANSI colours, breaking piped output. It lacks sorting control, verbose mode, and shell completions. These are the gaps most likely to frustrate real users.

## Scope

- Output summary after non-dry-run operations.
- `--no-color` flag and automatic detection of non-TTY output.
- `--verbose` mode showing each file move.
- Sorting options (alphabetical, by size, by date) before distribution.
- `--output-dir` flag for specifying a different destination.
- Shell completions generation via clap.
- Progress indicator for large operations.

## Child Tasks

- FEAT-001: Output summary for non-dry-run operations
- FEAT-002: --no-color flag and TTY detection
- FEAT-003: --verbose mode
- FEAT-004: Sorting options (--sort)
- FEAT-005: --output-dir flag
- FEAT-006: Shell completions generation
