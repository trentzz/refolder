# FEAT-001: Output summary for non-dry-run operations

**Epic**: FEAT (v0.2.0 Feature Additions)
**Priority**: high
**Depends on**: QUAL-003
**Status**: todo

## Goal

After a successful non-dry-run operation, print a summary showing: number of files moved, number of folders created, and the distribution. Currently the tool is completely silent on success, leaving users uncertain whether anything happened.

## Success Criteria

- [ ] Non-dry-run operations print a summary to stdout after completion.
- [ ] Summary includes: total files moved, total folders created/used, files per folder breakdown.
- [ ] Summary is visually distinct from dry-run output (no tree view, just a concise report).
- [ ] Existing dry-run output is unchanged.
- [ ] Tests verify summary output is produced.
- [ ] `/update` has been run after changes.

## Steps

1. Track move counts during the move loop in `run()`.
2. After all moves complete, print a summary block.
3. Add a test that captures stdout and verifies the summary.

## Notes

Consider reusing the tree format from dry-run for consistency, or keep it simpler. User preference may vary.
