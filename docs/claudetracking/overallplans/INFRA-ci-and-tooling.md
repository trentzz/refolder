# Epic: INFRA — CI and Tooling

## Goal

Set up continuous integration and ensure the project is ready for publishing to crates.io.

## Motivation

There is no CI configuration. The project has no `.gitignore` for common Rust artefacts (though `target/` appears untracked). No GitHub Actions workflow exists for running fmt, clippy, and tests on PRs.

## Scope

- GitHub Actions CI workflow (fmt, clippy, test, audit).
- Version bump to 0.2.0 when features are complete.

## Child Tasks

- INFRA-001: Add GitHub Actions CI workflow
- INFRA-002: Prepare 0.2.0 release metadata
