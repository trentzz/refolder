# refolder

`refolder` is a small Rust CLI that moves files matching a glob pattern into an equal number of subfolders.

## Install

You can install from a git repo with:

```bash
cargo install --git https://github.com/trentzz/refolder.git
```

## Usage

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

### Safety

A dry run is available for verification.

Dry run:

```bash
refolder . -m "*.log" -s 2 --dry-run
```

`--force` will overwrite files in destination if necessary. Without `--force`, existing destination files cause an error.

## Example

```bash
refolder /path/to --matching "*.txt" --subfolders 4 --prefix example --suffix numbers --recursive
```

Resulting folders will be:

```text
/path/to/example-1
/path/to/example-2
/path/to/example-3
/path/to/example-4
```

Files will be distributed as evenly as possible.

## Behavior notes

If files are already in subfolders that match the prefix and one of the -i indices (e.g. example-1), refolder will treat these as sources and will first collect their files to re-shuffle when re-distributing to a new number of subfolders. This allows "redoing" with a different --subfolders count.

The distribution ensures the number of files in any two target folders differ by at most 1.

Moves are attempted with fs::rename and will fall back to copy-and-remove if rename fails (e.g. across filesystems).

## More Examples

Split *.jpg files into 3 folders with prefix photos and numeric suffixes:

```bash
refolder . -m "*.jpg" -s 3 -p photos --suffix numbers -r
```

Dry run output

```bash
refolder . --matching '*.txt' --prefix simple --dry-run --subfolders 4`
Would create folder: ./simple-1
Would move: ./file1.txt -> ./simple-1/file1.txt
Would move: ./file10.txt -> ./simple-1/file10.txt
Would move: ./file11.txt -> ./simple-1/file11.txt
Would create folder: ./simple-2
Would move: ./file12.txt -> ./simple-2/file12.txt
Would move: ./file2.txt -> ./simple-2/file2.txt
Would move: ./file3.txt -> ./simple-2/file3.txt
Would create folder: ./simple-3
Would move: ./file4.txt -> ./simple-3/file4.txt
Would move: ./file5.txt -> ./simple-3/file5.txt
Would move: ./file6.txt -> ./simple-3/file6.txt
Would create folder: ./simple-4
Would move: ./file7.txt -> ./simple-4/file7.txt
Would move: ./file8.txt -> ./simple-4/file8.txt
Would move: ./file9.txt -> ./simple-4/file9.txt
```
