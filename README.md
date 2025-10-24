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
A CLI tool that redistributes files matching a pattern into evenly sized subfolders.

Usage: refolder [OPTIONS] --subfolders <SUBFOLDERS> <PATH>

Arguments:
  <PATH>  Path to the directory to search

Options:
  -m, --matching <MATCHING>      Glob pattern for matching files (shell-style). Default: "*" [default: *]
  -s, --subfolders <SUBFOLDERS>  Number of subfolders to split into
  -p, --prefix <PREFIX>          Prefix for created subfolders. Default: "group" [default: group]
      --suffix <SUFFIX>          Suffix style: numbers | letters | none [default: numbers]
  -r, --recursive                Recurse into subdirectories
      --dry-run                  Print actions without performing them
  -f, --force                    Overwrite existing files/folders in destination
  -h, --help                     Print help
  -V, --version                  Print version
```

> [!NOTE]
> `--force` will overwrite files in destination if necessary. Without `--force`, existing destination files cause an error.

## Examples

### Simple usage

```bash
refolder "/path/to/files" --matching "*.txt" --subfolders 4 --prefix "example"
```

Resulting folders will be:

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

Files will be distributed as evenly as possible.

### Dry run

```bash
$ refolder . --matching '*.txt' --prefix example --subfolders 4 --recursive --suffix letters --dry-run`
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

## Behavior notes

If files are already in subfolders that match the prefix and one of the -i indices (e.g. example-1), refolder will treat these as sources and will first collect their files to re-shuffle when re-distributing to a new number of subfolders. This allows "redoing" with a different --subfolders count.

The distribution ensures the number of files in any two target folders differ by at most 1.

Moves are attempted with fs::rename and will fall back to copy-and-remove if rename fails (e.g. across filesystems).
