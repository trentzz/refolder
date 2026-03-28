# Open Questions

## 1. --suffix none with multiple subfolders

Should `--suffix none --subfolders 3` be an error? Currently it silently puts all files in one folder. See `needs-review/suffix-none-bug.md`.

## 2. Progress indicator scope

Should v0.2.0 include a progress indicator for large operations, or defer it? See `needs-review/progress-indicator.md`.

## 3. Redo prefix matching precision

The redo logic matches any folder starting with the prefix string, which can match unrelated folders. Should this be fixed in v0.2.0? See `needs-review/redo-logic-prefix-matching.md`.
