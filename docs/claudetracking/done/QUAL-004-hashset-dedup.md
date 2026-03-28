# QUAL-004: Use HashSet for file deduplication in collect_files

**Epic**: QUAL (Code Quality and Lint Fixes)
**Priority**: medium
**Depends on**: QUAL-001
**Status**: done

## Goal

Replace the `O(n)` linear search `!files.contains(&p)` on line 197 of `lib.rs` with a `HashSet` for `O(1)` lookups. The current code is `O(n^2)` for directories with many files.

## Success Criteria

- [x] A `HashSet<PathBuf>` tracks seen files during collection.
- [x] The `files.contains(&p)` call is replaced with a HashSet lookup.
- [x] The final `Vec<PathBuf>` is still sorted before return.
- [x] All tests pass.
- [ ] `/update` has been run after changes.

## Steps

1. Add `use std::collections::HashSet;` to `lib.rs`.
2. Create a `HashSet<PathBuf>` alongside the `files` vec in `collect_files`.
3. Insert into both the set and the vec when adding files.
4. Use `set.contains()` instead of `files.contains()` for dedup checks.
5. Run tests.

## Notes

For typical usage (hundreds of files), this is not a bottleneck. But it is a correctness-of-complexity issue and trivial to fix.
