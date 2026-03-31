# refolder

`refolder` is a small Rust CLI that moves files matching a glob pattern into an equal number of subfolders.

## Install

You can install from a git repo with:

```bash
cargo install --git https://github.com/trentzz/refolder.git
```

Or once published:

```bash
cargo install refolder
```

## Usage

```bash
refolder "/path/to/files" --matching "*.txt" --subfolders 3 --prefix "example"
```

```text
Move matching files into equally-sized subfolders

Usage: refolder [OPTIONS] --subfolders <SUBFOLDERS> <PATH>

Arguments:
  <PATH>  Path to the directory to search

Options:
  -m, --matching <MATCHING>        Glob pattern for matching files (shell-style) [default: *]
  -s, --subfolders <SUBFOLDERS>    Number of subfolders to split into
  -p, --prefix <PREFIX>            Prefix for created subfolders [default: group]
      --suffix <SUFFIX>            Suffix style: numbers | letters | none [default: numbers]
  -o, --output-dir <PATH>          Directory where subfolders are created. Defaults to the source path
      --sort <MODE>                Sort order before distribution: name | none | size | size-desc [default: name]
  -r, --recursive                  Recurse into subdirectories
      --dry-run                    Print actions without performing them
  -f, --force                      Overwrite existing files/folders in destination
  -v, --verbose                    Print each file move to stderr as it happens
      --no-color                   Suppress all ANSI colour codes in output
  -h, --help                       Print help
  -V, --version                    Print version
```

> [!NOTE]
> `--force` will overwrite files in the destination if necessary. Without `--force`, an existing destination file causes an error.

## Examples

### Simple usage

```bash
refolder "/path/to/files" --matching "*.txt" --subfolders 4 --prefix "example"
```

Resulting folders:

```text
.
├── example-1
│   ├── file10.txt
│   ├── file11.txt
│   └── file1.txt
├── example-2
│   ├── file12.txt
│   ├── file2.txt
│   └── file3.txt
├── example-3
│   ├── file4.txt
│   ├── file5.txt
│   └── file6.txt
└── example-4
    ├── file7.txt
    ├── file8.txt
    └── file9.txt
```

Files are distributed as evenly as possible.

### Dry run

```bash
$ refolder . --matching '*.txt' --prefix example --subfolders 4 --recursive --suffix letters --dry-run
.
├── example-a
│   ├── file1.txt
│   ├── file10.txt
│   └── file11.txt
├── example-b
│   ├── file12.txt
│   ├── file2.txt
│   └── file3.txt
├── example-c
│   ├── file4.txt
│   ├── file5.txt
│   └── file6.txt
└── example-d
    ├── file7.txt
    ├── file8.txt
    └── file9.txt

Summary:
  Total folders: 4
  Total files:   12
  Mode:          dry-run (no changes made)
```

### Output to a different directory

```bash
refolder /data/raw --matching "*.csv" --subfolders 3 --output-dir /data/sorted
```

Subfolders are created inside `/data/sorted` rather than `/data/raw`. The output directory must already exist.

### Sorting files before distribution

```bash
refolder /data/raw --matching "*.bin" --subfolders 4 --sort size
```

Files are sorted smallest-first before being distributed. Use `size-desc` for largest-first, `name` for alphabetical (the default), or `none` to use filesystem order.

## Behaviour notes

- After a successful run, a summary is printed: e.g. `Moved 42 files into 5 folders (5 created).`
- Dry-run mode prints the planned folder tree and a summary without moving any files.
- If files already sit in subfolders that match the prefix and suffix pattern (e.g. `example-1`), refolder treats those as sources and collects their files before redistributing. This lets you re-run with a different `--subfolders` count.
- The distribution ensures any two target folders differ in file count by at most 1.
- Moves use `fs::rename` and fall back to copy-and-remove if the source and destination are on different filesystems.
- Colour output is enabled when stdout is a TTY. Pass `--no-color` or set the `NO_COLOR` environment variable to suppress it.
- `--suffix none` with `--subfolders` greater than 1 returns an error, since all files would land in a single folder with no way to distinguish them.
