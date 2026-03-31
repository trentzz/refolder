# FEAT-004: Sorting options (--sort)

**Epic**: FEAT (v0.2.0 Feature Additions)
**Priority**: medium
**Depends on**: QUAL-003
**Status**: todo

## Goal

Add a `--sort` flag that controls how files are ordered before distribution into buckets. Current behaviour sorts alphabetically by path. Users may want to sort by file size or modification date to get more even distributions by size.

## Success Criteria

- [ ] A `--sort` flag is available with values: `name` (default, current behaviour), `size`, `date`, `none`.
- [ ] `size` sorts by file size (ascending).
- [ ] `date` sorts by last modification time (ascending).
- [ ] `none` preserves filesystem iteration order.
- [ ] The default behaviour (alphabetical) is unchanged.
- [ ] Tests verify at least one non-default sort order.
- [ ] `/update` has been run after changes.

## Steps

1. Add `--sort` flag to Args and Config.
2. After collecting files, sort them according to the chosen strategy.
3. Add tests using files with known sizes or timestamps.

## Notes

Sorting by size before round-robin distribution does not guarantee even total sizes per bucket. That would require a bin-packing algorithm. This task only controls the order of distribution, which is still useful for grouping similar files.
