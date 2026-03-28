# Epic: TEST — Expanded Test Coverage

## Goal

Cover untested code paths and edge cases. The current test suite has six tests that cover the happy path but miss error handling, flag interactions, and boundary conditions.

## Motivation

The `--force` flag overwrite behaviour, `--recursive` with nested directories, `--suffix none` with multiple subfolders (which would create duplicate folder names), dry-run output correctness, and error paths (non-existent directory, non-UTF-8 paths) are all untested. Without coverage here, regressions are likely as features are added.

## Scope

- Test `--force` overwrite behaviour.
- Test `--recursive` with nested directories.
- Test `--suffix none` edge case (duplicate folder names).
- Test error paths: non-existent path, non-directory path, zero subfolders.
- Test dry-run output content.

## Child Tasks

- TEST-001: Test --force flag behaviour
- TEST-002: Test --recursive with nested directories
- TEST-003: Test error paths and edge cases
