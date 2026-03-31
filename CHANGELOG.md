# Changelog

## [0.2.0] - 2026-03-29

### Added

- Output summary after a successful run (e.g. "Moved 42 files into 5 folders (5 created).").
- `--no-color` flag and `NO_COLOR` environment variable support to disable coloured output.
- `--verbose` / `-v` flag to print each file move as it happens.
- `--sort <MODE>` flag with `name`, `none`, `size`, and `size-desc` modes.
- `--output-dir <PATH>` / `-o` flag to specify a destination directory separate from the source.
- GitHub Actions CI workflow running fmt, clippy, and tests on each push.

### Fixed

- `--suffix none` combined with `--subfolders > 1` now returns a clear error instead of silently misbehaving.
- Prefix matching now requires a separator, preventing a prefix like "group" from matching "grouper".
- Logic bug where files could be moved twice when the subfolder count reached 10 or more, caused by a `starts_with` string collision.
- O(n²) deduplication replaced with a `HashSet` for O(1) performance.

## [0.1.0] - initial release
