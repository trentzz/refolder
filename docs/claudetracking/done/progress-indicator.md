# Needs Review: Progress indicator for large operations

## Finding

For directories with thousands of files, the tool provides no feedback during operation. A progress bar or counter would improve the experience.

## Options

1. **indicatif crate**: full progress bar with ETA. Adds a dependency.
2. **Simple counter**: print "moved N/M files..." to stderr periodically. No dependency.
3. **Skip for v0.2.0**: the tool is fast enough for most use cases. Defer to a later version.

## Recommendation

Option 2 (simple counter) is lightweight and dependency-free. But this may not be worth the effort for v0.2.0 if the typical use case is hundreds of files, not millions.

## Action needed

User decision on whether to include this in v0.2.0 scope.
