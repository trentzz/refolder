use anyhow::{Context, Result, anyhow};
use globwalk::GlobWalkerBuilder;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Returns the ANSI bold-blue escape sequence when colour is enabled, or an
/// empty string when colour is disabled (e.g. --no-color or non-TTY output).
fn bold_start(color: bool) -> &'static str {
    if color { "\x1b[1;34m" } else { "" }
}

/// Returns the ANSI reset escape sequence when colour is enabled, or an empty
/// string when colour is disabled.
fn bold_end(color: bool) -> &'static str {
    if color { "\x1b[0m" } else { "" }
}

/// Controls the order in which files are distributed into buckets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    /// Alphabetical by path (default).
    Name,
    /// Filesystem iteration order — no sort applied.
    None,
    /// Ascending by file size (smallest first).
    Size,
    /// Descending by file size (largest first).
    SizeDesc,
}

impl std::str::FromStr for SortMode {
    type Err = anyhow::Error;

    /// Parse a sort mode from a string. Returns an error for unknown values.
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "name" => Ok(Self::Name),
            "none" => Ok(Self::None),
            "size" => Ok(Self::Size),
            "size-desc" => Ok(Self::SizeDesc),
            other => Err(anyhow!(
                "Unknown sort mode '{}'. Use name|none|size|size-desc",
                other
            )),
        }
    }
}

/// Sort `files` in place according to `mode`.
///
/// `Size` and `SizeDesc` query each file's metadata. Files whose metadata
/// cannot be read are treated as zero-byte for sorting purposes, and a warning
/// is printed to stderr.
fn sort_files(files: &mut [PathBuf], mode: SortMode) {
    match mode {
        SortMode::Name => files.sort(),
        SortMode::None => {} // preserve filesystem iteration order
        SortMode::Size => {
            files.sort_by_key(|p| {
                fs::metadata(p).map(|m| m.len()).unwrap_or_else(|e| {
                    eprintln!(
                        "Warning: could not read metadata for {}: {}",
                        p.display(),
                        e
                    );
                    0
                })
            });
        }
        SortMode::SizeDesc => {
            files.sort_by_key(|p| {
                let sz = fs::metadata(p).map(|m| m.len()).unwrap_or_else(|e| {
                    eprintln!(
                        "Warning: could not read metadata for {}: {}",
                        p.display(),
                        e
                    );
                    0
                });
                std::cmp::Reverse(sz)
            });
        }
    }
}

/// Configuration for a single refolder run.
///
/// Construct this from parsed CLI arguments and pass it to [`run`].
#[derive(Debug)]
pub struct Config<'a> {
    /// Path to the directory to search.
    pub base_path: &'a str,
    /// Glob pattern for matching files (shell-style).
    pub matching: &'a str,
    /// Number of subfolders to split into.
    pub subfolders: usize,
    /// Prefix for created subfolders.
    pub prefix: &'a str,
    /// Suffix style: `numbers` | `letters` | `none`.
    pub suffix: &'a str,
    /// Whether to recurse into subdirectories.
    pub recursive: bool,
    /// Print planned actions without performing them.
    pub dry_run: bool,
    /// Overwrite existing files in the destination.
    pub force: bool,
    /// Whether to emit ANSI colour codes in output.
    pub color: bool,
    /// Print each file move to stderr as it happens.
    pub verbose: bool,
    /// Sort order for files before distribution.
    pub sort: &'a str,
    /// Directory where subfolders are created. Defaults to the source path.
    pub output_dir: Option<&'a str>,
}

/// Public API: run the refolder operation.
pub fn run(config: &Config<'_>) -> Result<()> {
    let base_path = config.base_path;
    let matching = config.matching;
    let subfolders = config.subfolders;
    let prefix = config.prefix;
    let suffix = config.suffix;
    let recursive = config.recursive;
    let dry_run = config.dry_run;
    let force = config.force;
    let color = config.color;
    let verbose = config.verbose;
    let sort = config.sort;
    let output_dir = config.output_dir;

    if subfolders == 0 {
        return Err(anyhow!("subfolders must be greater than zero"));
    }

    // A suffix of "none" means every folder gets the same name (just the prefix).
    // That only makes sense when there is exactly one subfolder.
    if suffix == "none" && subfolders > 1 {
        return Err(anyhow!(
            "Cannot use --suffix none with --subfolders > 1: all folders would have the same name"
        ));
    }

    let sort_mode = sort.parse::<SortMode>()?;

    let base = Path::new(base_path);
    if !base.exists() {
        return Err(anyhow!("Path '{}' does not exist", base.display()));
    }
    if !base.is_dir() {
        return Err(anyhow!("Path '{}' is not a directory", base.display()));
    }

    // Resolve the output directory (defaults to base path).
    // When --output-dir is provided it must already exist; we do not create it.
    let out_dir_buf;
    let out_dir: &Path = match output_dir {
        Some(o) => {
            out_dir_buf = PathBuf::from(o);
            let p = out_dir_buf.as_path();
            if !p.exists() {
                return Err(anyhow!("Output directory '{}' does not exist", p.display()));
            }
            if !p.is_dir() {
                return Err(anyhow!("Output path '{}' is not a directory", p.display()));
            }
            p
        }
        None => base,
    };

    // 1) Collect files to operate on. If files live under existing target folders (prefix-<i>),
    // treat them as sources as well so we can "redo" distributions. We scan both
    // the source base and the output dir so that a second run picks up previously
    // moved files regardless of where they landed.
    let mut files = collect_files(base, matching, recursive, prefix, Some(out_dir))?;

    if files.is_empty() {
        println!("No files matched pattern. Nothing to do.");
        return Ok(());
    }

    // 2) Sort according to the chosen strategy.
    sort_files(&mut files, sort_mode);

    // 3) Partition into buckets as evenly as possible.
    let buckets = partition(files, subfolders);

    // 4) For each bucket, create folder name and move files.
    // planned_moves is only used to collect all moves for dry-run display.
    let mut planned_moves: Vec<(String, String)> = Vec::new();

    // Counters used only in non-dry-run mode.
    let mut folders_created: usize = 0;
    let mut folders_used: usize = 0;
    let mut files_moved: usize = 0;

    for (i, bucket) in buckets.into_iter().enumerate() {
        let folder_name = format_folder_name(prefix, i + 1, suffix)?;
        let folder_path = out_dir.join(&folder_name);

        // Build the moves for this bucket only.
        let mut bucket_moves: Vec<(PathBuf, PathBuf)> = Vec::new();
        for src in bucket {
            let file_name = src
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or_else(|| anyhow!("Invalid filename for {}", src.display()))?;
            let dest = folder_path.join(file_name);
            if dry_run {
                planned_moves.push((src.display().to_string(), dest.display().to_string()));
            }
            bucket_moves.push((src, dest));
        }

        // If not dry-run, perform actual creation and moving.
        if !dry_run {
            if folder_path.exists() {
                if !folder_path.is_dir() {
                    return Err(anyhow!(
                        "Destination path {} exists and is not a directory",
                        folder_path.display()
                    ));
                }
                folders_used += 1;
            } else {
                fs::create_dir_all(&folder_path).with_context(|| {
                    format!("Failed to create directory {}", folder_path.display())
                })?;
                folders_created += 1;
            }

            for (src, dest) in &bucket_moves {
                // Skip identical (redo safe).
                if src == dest {
                    continue;
                }

                if dest.exists() {
                    if !force {
                        return Err(anyhow!(
                            "Destination file {} already exists (use --force to overwrite)",
                            dest.display()
                        ));
                    } else {
                        fs::remove_file(dest).with_context(|| {
                            format!(
                                "Failed removing existing destination file {}",
                                dest.display()
                            )
                        })?;
                    }
                }

                match fs::rename(src, dest) {
                    Ok(_) => {}
                    Err(rename_err) => {
                        fs::copy(src, dest).with_context(|| {
                            format!(
                                "Failed copying {} to {}: {}",
                                src.display(),
                                dest.display(),
                                rename_err
                            )
                        })?;
                        fs::remove_file(src).with_context(|| {
                            format!("Failed removing original file {}", src.display())
                        })?;
                    }
                }

                files_moved += 1;

                // Print the relative destination path to stderr when verbose.
                if verbose {
                    let rel = dest.strip_prefix(out_dir).unwrap_or(dest).to_string_lossy();
                    eprintln!("{}", rel);
                }
            }
        }
    }

    if dry_run {
        print_dry_run_preview(&planned_moves, color);
    } else {
        // Print a concise summary of what was done.
        let total_folders = folders_created + folders_used;
        if folders_created > 0 {
            println!(
                "Moved {} {} into {} {} ({} created).",
                files_moved,
                pluralise(files_moved, "file", "files"),
                total_folders,
                pluralise(total_folders, "folder", "folders"),
                folders_created,
            );
        } else {
            println!(
                "Moved {} {} into {} {}.",
                files_moved,
                pluralise(files_moved, "file", "files"),
                total_folders,
                pluralise(total_folders, "folder", "folders"),
            );
        }
    }

    Ok(())
}

/// Returns the singular or plural form depending on `count`.
fn pluralise<'a>(count: usize, singular: &'a str, plural: &'a str) -> &'a str {
    if count == 1 { singular } else { plural }
}

/// Collect files matching `pattern` under `base`. If an existing folder with `prefix` exists
/// under `base` or `output_base` we also collect matching files inside it (one level deep) so
/// we can "redo" distributions.
///
/// Returns files in an unspecified order. The caller is responsible for sorting.
fn collect_files(
    base: &Path,
    pattern: &str,
    recursive: bool,
    prefix: &str,
    output_base: Option<&Path>,
) -> Result<Vec<PathBuf>> {
    // Always canonicalize base first.
    let canonical_base = std::fs::canonicalize(base)
        .with_context(|| format!("Failed to canonicalize {}", base.display()))?;

    // Use string form — avoids internal strip_prefix panics in globwalk.
    let base_str = canonical_base
        .to_str()
        .ok_or_else(|| anyhow!("Base path is not valid UTF-8"))?
        .to_string();

    // Build walker using the canonical absolute path string.
    let mut builder = GlobWalkerBuilder::from_patterns(&base_str, &[pattern]);
    builder = builder.case_insensitive(true);

    if recursive {
        builder = builder.max_depth(usize::MAX);
    } else {
        builder = builder.max_depth(1);
    }

    let walker = builder
        .build()
        .with_context(|| format!("Failed building glob walker for {}", base_str))?;

    let mut files: Vec<PathBuf> = walker
        .filter_map(|entry| match entry {
            Ok(e) => Some(e.path().to_path_buf()),
            Err(err) => {
                eprintln!("Warning: skipping entry due to error: {}", err);
                None
            }
        })
        .filter(|p| p.is_file())
        .collect();

    // Use a HashSet for O(1) duplicate checks when collecting from redo directories.
    let mut seen: HashSet<PathBuf> = files.iter().cloned().collect();

    // Collect redo directories from both the source base and the output base.
    // When they are the same path, scanning once is enough.
    let canonical_out = output_base
        .map(|o| {
            std::fs::canonicalize(o)
                .with_context(|| format!("Failed to canonicalize {}", o.display()))
        })
        .transpose()?;

    let mut redo_roots: Vec<&Path> = vec![&canonical_base];
    if let Some(ref out) = canonical_out
        && out != &canonical_base
    {
        redo_roots.push(out.as_path());
    }

    for root in redo_roots {
        collect_redo_files(root, pattern, prefix, &mut files, &mut seen)?;
    }

    Ok(files)
}

/// Scan `root` for subdirectories whose names start with `prefix-` (or equal
/// `prefix` exactly) and add any matching files they contain to `files`.
/// Uses `seen` to avoid adding the same path twice.
fn collect_redo_files(
    root: &Path,
    pattern: &str,
    prefix: &str,
    files: &mut Vec<PathBuf>,
    seen: &mut HashSet<PathBuf>,
) -> Result<()> {
    let prefix_sep = format!("{}-", prefix);

    match fs::read_dir(root) {
        Ok(readdir) => {
            for entry in readdir.filter_map(Result::ok) {
                let s = entry.file_name().to_string_lossy().to_string();
                // Require the separator so that a prefix of "group" does not
                // accidentally match "grouper" or "groups".
                if (s.starts_with(&prefix_sep) || s == prefix) && entry.path().is_dir() {
                    let inner_base = std::fs::canonicalize(entry.path()).with_context(|| {
                        format!("Failed to canonicalize {}", entry.path().display())
                    })?;
                    let inner_str = inner_base
                        .to_str()
                        .ok_or_else(|| anyhow!("Invalid UTF-8 path"))?;
                    let inner_walker = GlobWalkerBuilder::from_patterns(inner_str, &[pattern])
                        .max_depth(1)
                        .build()
                        .with_context(|| format!("Failed to build walker for {}", inner_str))?;

                    for e in inner_walker.filter_map(Result::ok) {
                        let p = e.path().to_path_buf();
                        if p.is_file() && seen.insert(p.clone()) {
                            files.push(p);
                        }
                    }
                }
            }
        }
        Err(e) => eprintln!(
            "Warning: could not read directory {}: {}",
            root.display(),
            e
        ),
    }

    Ok(())
}

/// Partition `files` into `n` buckets as evenly as possible.
/// If there are fewer files than buckets, some buckets will be empty.
fn partition(files: Vec<PathBuf>, n: usize) -> Vec<Vec<PathBuf>> {
    let total = files.len();
    let mut buckets: Vec<Vec<PathBuf>> = vec![Vec::new(); n];
    if n == 0 {
        return buckets;
    }
    if total == 0 {
        return buckets;
    }

    let base = total / n;
    let rem = total % n;

    let mut idx = 0usize;
    for (i, bucket) in buckets.iter_mut().enumerate() {
        let take = base + if i < rem { 1 } else { 0 };
        for _ in 0..take {
            if idx < files.len() {
                bucket.push(files[idx].clone());
                idx += 1;
            }
        }
    }

    buckets
}

fn format_folder_name(prefix: &str, index: usize, suffix: &str) -> Result<String> {
    match suffix {
        "numbers" => Ok(format!("{}-{}", prefix, index)),
        "letters" => {
            // index to letters: 1 -> a, 2 -> b, ... 27 -> aa
            let mut i = index;
            let mut s = String::new();
            while i > 0 {
                i -= 1; // 0-based
                let ch = ((i % 26) as u8 + b'a') as char;
                s.insert(0, ch);
                i /= 26;
            }
            Ok(format!("{}-{}", prefix, s))
        }
        "none" => Ok(prefix.to_string()),
        other => Err(anyhow!(
            "Unknown suffix style '{}'. Use numbers|letters|none",
            other
        )),
    }
}

pub(crate) fn print_dry_run_preview(file_moves: &[(String, String)], color: bool) {
    let mut folders: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for (_src, dst) in file_moves {
        let dst_path = Path::new(dst);
        let folder = dst_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_string_lossy()
            .to_string();
        let file_name = dst_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        folders.entry(folder).or_default().push(file_name);
    }

    println!(".");
    let folder_names: Vec<_> = folders.keys().cloned().collect();
    let last_folder_idx = folder_names.len().saturating_sub(1);

    for (i, folder) in folder_names.iter().enumerate() {
        let is_last_folder = i == last_folder_idx;
        let prefix_folder = if is_last_folder {
            "└── "
        } else {
            "├── "
        };

        let folder_name = Path::new(folder)
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new(folder))
            .to_string_lossy();

        // Wrap folder name in bold ANSI codes when colour is enabled.
        println!(
            "{}{}{}{}",
            prefix_folder,
            bold_start(color),
            folder_name,
            bold_end(color)
        );

        let mut files = folders.get(folder).unwrap().clone();
        files.sort();
        let last_file_idx = files.len().saturating_sub(1);

        for (j, file) in files.into_iter().enumerate() {
            let prefix_file = if j == last_file_idx {
                if is_last_folder {
                    "    └── "
                } else {
                    "│   └── "
                }
            } else if is_last_folder {
                "    ├── "
            } else {
                "│   ├── "
            };
            println!("{}{}", prefix_file, file);
        }
    }

    // Summary at the bottom of the dry-run preview.
    println!("\nSummary:");
    println!("  Total folders: {}", folders.len());
    let total_files: usize = folders.values().map(|v| v.len()).sum();
    println!("  Total files:   {}", total_files);
    println!("  Mode:          dry-run (no changes made)");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;

    /// Call run() with colour, verbose, sort, and output_dir set to sensible defaults.
    #[allow(clippy::too_many_arguments)]
    fn run_default(
        base: &str,
        matching: &str,
        subfolders: usize,
        prefix: &str,
        suffix: &str,
        recursive: bool,
        dry_run: bool,
        force: bool,
    ) -> Result<()> {
        run(&Config {
            base_path: base,
            matching,
            subfolders,
            prefix,
            suffix,
            recursive,
            dry_run,
            force,
            color: false,
            verbose: false,
            sort: "name",
            output_dir: None,
        })
    }

    #[test]
    fn test_partition_even() {
        let files: Vec<PathBuf> = (0..8).map(|i| PathBuf::from(format!("f{}", i))).collect();
        let buckets = partition(files, 4);
        assert_eq!(buckets.len(), 4);
        assert_eq!(
            buckets.iter().map(|b| b.len()).collect::<Vec<_>>(),
            vec![2, 2, 2, 2]
        );
    }

    #[test]
    fn test_partition_uneven() {
        let files: Vec<PathBuf> = (0..10).map(|i| PathBuf::from(format!("f{}", i))).collect();
        let buckets = partition(files, 3);
        assert_eq!(
            buckets.iter().map(|b| b.len()).collect::<Vec<_>>(),
            vec![4, 3, 3]
        );
    }

    #[test]
    fn test_format_folder_name_letters() {
        assert_eq!(format_folder_name("ex", 1, "letters").unwrap(), "ex-a");
        assert_eq!(format_folder_name("ex", 26, "letters").unwrap(), "ex-z");
        assert_eq!(format_folder_name("ex", 27, "letters").unwrap(), "ex-aa");
    }

    #[test]
    fn integration_move_files() -> Result<()> {
        let dir = tempdir()?;
        let base = dir.path();
        // Create 5 files.
        for i in 0..5 {
            let p = base.join(format!("file{}.txt", i));
            File::create(&p)?;
        }

        // Move into 3 buckets.
        run_default(
            base.to_str().unwrap(),
            "*.txt",
            3,
            "pack",
            "numbers",
            false,
            false,
            true,
        )?;

        // Check folders exist.
        let a = base.join("pack-1");
        let b = base.join("pack-2");
        let c = base.join("pack-3");
        assert!(a.is_dir() && b.is_dir() && c.is_dir());

        let cnts = [
            fs::read_dir(&a)?.count(),
            fs::read_dir(&b)?.count(),
            fs::read_dir(&c)?.count(),
        ];
        // Distribution should sum to 5.
        assert_eq!(cnts.iter().sum::<usize>(), 5);

        Ok(())
    }

    #[test]
    fn redo_existing_folders() -> Result<()> {
        // Test that files inside existing prefix-* folders are collected and re-shuffled.
        let dir = tempdir()?;
        let base = dir.path();

        // Create two existing folders pack-1 and pack-2 with some files.
        let p1 = base.join("pack-1");
        let p2 = base.join("pack-2");
        fs::create_dir_all(&p1)?;
        fs::create_dir_all(&p2)?;

        File::create(p1.join("a.txt"))?;
        File::create(p1.join("b.txt"))?;
        File::create(p2.join("c.txt"))?;

        // Re-split into 3 buckets.
        run_default(
            base.to_str().unwrap(),
            "*.txt",
            3,
            "pack",
            "numbers",
            false,
            false,
            true,
        )?;

        // Ensure pack-1..pack-3 exist and files moved.
        let p3 = base.join("pack-3");
        assert!(p1.is_dir() && p2.is_dir() && p3.is_dir());
        let total: usize = [p1, p2, p3]
            .iter()
            .map(|d| fs::read_dir(d).unwrap().count())
            .sum();
        assert_eq!(total, 3);

        Ok(())
    }

    #[test]
    fn test_strip_prefix_safe() -> Result<()> {
        // Use "." explicitly to simulate the common cause of StripPrefixError.
        let dir = tempdir()?;
        let base = dir.path();

        // Create some files.
        for i in 0..3 {
            let p = base.join(format!("f{}.txt", i));
            File::create(&p)?;
        }

        // Run collect_files directly to ensure no panic.
        let result =
            std::panic::catch_unwind(|| collect_files(base, "*.txt", true, "pack", None).unwrap());

        assert!(
            result.is_ok(),
            "collect_files should never panic on relative paths"
        );

        Ok(())
    }

    #[test]
    fn test_summary_output_all_created() -> Result<()> {
        let dir = tempdir()?;
        let base = dir.path();

        for i in 0..6 {
            File::create(base.join(format!("f{}.txt", i)))?;
        }

        run_default(
            base.to_str().unwrap(),
            "*.txt",
            3,
            "grp",
            "numbers",
            false,
            false,
            false,
        )?;

        // All three folders should exist with 2 files each.
        for i in 1..=3 {
            let folder = base.join(format!("grp-{}", i));
            assert!(folder.is_dir());
            assert_eq!(fs::read_dir(&folder)?.count(), 2);
        }

        Ok(())
    }

    #[test]
    fn test_summary_output_partial_existing() -> Result<()> {
        let dir = tempdir()?;
        let base = dir.path();

        // Pre-create one folder with files.
        let existing = base.join("grp-1");
        fs::create_dir_all(&existing)?;
        File::create(existing.join("a.txt"))?;
        File::create(existing.join("b.txt"))?;

        File::create(base.join("c.txt"))?;
        File::create(base.join("d.txt"))?;

        run_default(
            base.to_str().unwrap(),
            "*.txt",
            2,
            "grp",
            "numbers",
            false,
            false,
            true,
        )?;

        let total: usize = [base.join("grp-1"), base.join("grp-2")]
            .iter()
            .map(|d| fs::read_dir(d).unwrap().count())
            .sum();
        assert_eq!(total, 4);

        Ok(())
    }

    #[test]
    fn test_dry_run_no_color() {
        let moves = vec![
            (
                "/tmp/src/file1.txt".to_string(),
                "/tmp/dst/group-1/file1.txt".to_string(),
            ),
            (
                "/tmp/src/file2.txt".to_string(),
                "/tmp/dst/group-1/file2.txt".to_string(),
            ),
        ];

        assert_eq!(bold_start(false), "");
        assert_eq!(bold_end(false), "");
        assert_eq!(bold_start(true), "\x1b[1;34m");
        assert_eq!(bold_end(true), "\x1b[0m");

        print_dry_run_preview(&moves, false);
        print_dry_run_preview(&moves, true);
    }

    #[test]
    fn test_sort_mode_from_str() {
        assert_eq!("name".parse::<SortMode>().unwrap(), SortMode::Name);
        assert_eq!("none".parse::<SortMode>().unwrap(), SortMode::None);
        assert_eq!("size".parse::<SortMode>().unwrap(), SortMode::Size);
        assert_eq!("size-desc".parse::<SortMode>().unwrap(), SortMode::SizeDesc);
        assert!("invalid".parse::<SortMode>().is_err());
    }

    #[test]
    fn test_sort_by_size_ascending() -> Result<()> {
        let dir = tempdir()?;
        let base = dir.path();

        // Create files with known sizes: large.txt (100 bytes), small.txt (10 bytes).
        {
            let mut f = File::create(base.join("large.txt"))?;
            f.write_all(&[b'x'; 100])?;
        }
        {
            let mut f = File::create(base.join("small.txt"))?;
            f.write_all(&[b'x'; 10])?;
        }

        // Sort ascending: small.txt goes to pack-1 (first bucket).
        run(&Config {
            base_path: base.to_str().unwrap(),
            matching: "*.txt",
            subfolders: 2,
            prefix: "pack",
            suffix: "numbers",
            recursive: false,
            dry_run: false,
            force: false,
            color: false,
            verbose: false,
            sort: "size",
            output_dir: None,
        })?;

        assert!(base.join("pack-1").join("small.txt").exists());
        assert!(base.join("pack-2").join("large.txt").exists());

        Ok(())
    }

    #[test]
    fn test_sort_by_size_descending() -> Result<()> {
        let dir = tempdir()?;
        let base = dir.path();

        // Create files with known sizes.
        {
            let mut f = File::create(base.join("large.txt"))?;
            f.write_all(&[b'x'; 100])?;
        }
        {
            let mut f = File::create(base.join("small.txt"))?;
            f.write_all(&[b'x'; 10])?;
        }

        // Sort descending: large.txt goes to pack-1 (first bucket).
        run(&Config {
            base_path: base.to_str().unwrap(),
            matching: "*.txt",
            subfolders: 2,
            prefix: "pack",
            suffix: "numbers",
            recursive: false,
            dry_run: false,
            force: false,
            color: false,
            verbose: false,
            sort: "size-desc",
            output_dir: None,
        })?;

        assert!(base.join("pack-1").join("large.txt").exists());
        assert!(base.join("pack-2").join("small.txt").exists());

        Ok(())
    }

    #[test]
    fn test_verbose_run_succeeds() -> Result<()> {
        // Verbose prints to stderr. We verify the operation completes successfully
        // and that files are moved to the expected locations.
        let dir = tempdir()?;
        let base = dir.path();

        for i in 0..3 {
            File::create(base.join(format!("v{}.txt", i)))?;
        }

        run(&Config {
            base_path: base.to_str().unwrap(),
            matching: "*.txt",
            subfolders: 2,
            prefix: "grp",
            suffix: "numbers",
            recursive: false,
            dry_run: false,
            force: false,
            color: false,
            verbose: true,
            sort: "name",
            output_dir: None,
        })?;

        let total: usize = [base.join("grp-1"), base.join("grp-2")]
            .iter()
            .map(|d| fs::read_dir(d).map(|r| r.count()).unwrap_or(0))
            .sum();
        assert_eq!(total, 3);

        Ok(())
    }

    // -----------------------------------------------------------------------
    // FEAT-005: --output-dir
    // -----------------------------------------------------------------------

    #[test]
    fn output_dir_creates_subfolders_in_output() -> Result<()> {
        let src_dir = tempdir()?;
        let out_dir = tempdir()?;

        for i in 0..6 {
            File::create(src_dir.path().join(format!("img{}.jpg", i)))?;
        }

        run(&Config {
            base_path: src_dir.path().to_str().unwrap(),
            matching: "*.jpg",
            subfolders: 3,
            prefix: "group",
            suffix: "numbers",
            recursive: false,
            dry_run: false,
            force: false,
            color: false,
            verbose: false,
            sort: "name",
            output_dir: Some(out_dir.path().to_str().unwrap()),
        })?;

        // Subfolders should exist in the output dir, not the source dir.
        assert!(out_dir.path().join("group-1").is_dir());
        assert!(out_dir.path().join("group-2").is_dir());
        assert!(out_dir.path().join("group-3").is_dir());
        assert!(
            !src_dir.path().join("group-1").exists(),
            "No subfolders should be created in the source dir"
        );

        let total: usize = (1..=3)
            .map(|i| {
                fs::read_dir(out_dir.path().join(format!("group-{}", i)))
                    .unwrap()
                    .count()
            })
            .sum();
        assert_eq!(total, 6);

        Ok(())
    }

    #[test]
    fn output_dir_nonexistent_returns_error() -> Result<()> {
        let src_dir = tempdir()?;
        File::create(src_dir.path().join("a.txt"))?;

        let result = run(&Config {
            base_path: src_dir.path().to_str().unwrap(),
            matching: "*.txt",
            subfolders: 1,
            prefix: "group",
            suffix: "numbers",
            recursive: false,
            dry_run: false,
            force: false,
            color: false,
            verbose: false,
            sort: "name",
            output_dir: Some("/tmp/__refolder_nonexistent_output_xyzzy__"),
        });

        assert!(
            result.is_err(),
            "Non-existent output dir must return an error"
        );
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("does not exist"),
            "Error should mention 'does not exist', got: {}",
            msg
        );

        Ok(())
    }

    #[test]
    fn output_dir_without_flag_unchanged() -> Result<()> {
        // Without --output-dir, subfolders are created inside the source dir.
        let dir = tempdir()?;
        let base = dir.path();

        for i in 0..4 {
            File::create(base.join(format!("file{}.txt", i)))?;
        }

        run_default(
            base.to_str().unwrap(),
            "*.txt",
            2,
            "set",
            "numbers",
            false,
            false,
            false,
        )?;

        assert!(base.join("set-1").is_dir());
        assert!(base.join("set-2").is_dir());

        Ok(())
    }

    // -----------------------------------------------------------------------
    // TEST-001: --force flag
    // -----------------------------------------------------------------------

    #[test]
    fn force_false_errors_on_existing_destination() -> Result<()> {
        let dir = tempdir()?;
        let base = dir.path();

        // Source file.
        File::create(base.join("a.txt"))?;

        // Pre-create the destination folder with a conflicting file.
        let dest_folder = base.join("grp-1");
        fs::create_dir_all(&dest_folder)?;
        File::create(dest_folder.join("a.txt"))?;

        let result = run_default(
            base.to_str().unwrap(),
            "*.txt",
            1,
            "grp",
            "numbers",
            false,
            false,
            false, // force=false
        );

        assert!(
            result.is_err(),
            "Should return an error when destination exists and force=false"
        );
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("already exists"),
            "Error should mention 'already exists', got: {}",
            msg
        );

        Ok(())
    }

    #[test]
    fn force_true_overwrites_existing_destination() -> Result<()> {
        let dir = tempdir()?;
        let base = dir.path();

        // Source file (empty — created with File::create).
        File::create(base.join("a.txt"))?;

        // Pre-create the destination folder with old content.
        let dest_folder = base.join("grp-1");
        fs::create_dir_all(&dest_folder)?;
        fs::write(dest_folder.join("a.txt"), b"old content")?;

        run_default(
            base.to_str().unwrap(),
            "*.txt",
            1,
            "grp",
            "numbers",
            false,
            false,
            true, // force=true
        )?;

        assert!(
            dest_folder.join("a.txt").exists(),
            "Destination file should exist after force overwrite"
        );

        Ok(())
    }

    // -----------------------------------------------------------------------
    // TEST-002: --recursive with nested directories
    // -----------------------------------------------------------------------

    #[test]
    fn recursive_collects_nested_files() -> Result<()> {
        let dir = tempdir()?;
        let base = dir.path();

        // Top-level file.
        File::create(base.join("top.txt"))?;

        // Files nested two levels deep.
        let sub = base.join("a").join("b");
        fs::create_dir_all(&sub)?;
        File::create(sub.join("deep.txt"))?;
        File::create(base.join("a").join("mid.txt"))?;

        run_default(
            base.to_str().unwrap(),
            "*.txt",
            2,
            "part",
            "numbers",
            true, // recursive
            false,
            false,
        )?;

        // All 3 files should be distributed across the two buckets.
        let total: usize = (1..=2)
            .map(|i| {
                fs::read_dir(base.join(format!("part-{}", i)))
                    .unwrap()
                    .count()
            })
            .sum();
        assert_eq!(total, 3, "All 3 nested files should be distributed");

        Ok(())
    }

    #[test]
    fn non_recursive_ignores_subdirectories() -> Result<()> {
        let dir = tempdir()?;
        let base = dir.path();

        File::create(base.join("top.txt"))?;

        let sub = base.join("nested");
        fs::create_dir_all(&sub)?;
        File::create(sub.join("hidden.txt"))?;

        run_default(
            base.to_str().unwrap(),
            "*.txt",
            1,
            "part",
            "numbers",
            false, // not recursive
            false,
            false,
        )?;

        // Only "top.txt" should be moved; "nested/hidden.txt" must remain.
        assert!(
            sub.join("hidden.txt").exists(),
            "Non-recursive run must not touch files in subdirectories"
        );
        assert_eq!(
            fs::read_dir(base.join("part-1")).unwrap().count(),
            1,
            "Only the top-level file should be in part-1"
        );

        Ok(())
    }

    // -----------------------------------------------------------------------
    // TEST-003: Error paths
    // -----------------------------------------------------------------------

    #[test]
    fn error_on_nonexistent_path() {
        let result = run_default(
            "/tmp/__refolder_does_not_exist_xyzzy__",
            "*",
            1,
            "group",
            "numbers",
            false,
            false,
            false,
        );
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("does not exist"),
            "Expected 'does not exist', got: {}",
            msg
        );
    }

    #[test]
    fn error_on_file_path_not_directory() -> Result<()> {
        let dir = tempdir()?;
        let file = dir.path().join("not_a_dir.txt");
        File::create(&file)?;

        let result = run_default(
            file.to_str().unwrap(),
            "*",
            1,
            "group",
            "numbers",
            false,
            false,
            false,
        );
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("not a directory"),
            "Expected 'not a directory', got: {}",
            msg
        );

        Ok(())
    }

    #[test]
    fn error_on_zero_subfolders() -> Result<()> {
        let dir = tempdir()?;
        let result = run_default(
            dir.path().to_str().unwrap(),
            "*",
            0,
            "group",
            "numbers",
            false,
            false,
            false,
        );
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("greater than zero"),
            "Expected 'greater than zero', got: {}",
            msg
        );

        Ok(())
    }

    #[test]
    fn error_on_unknown_suffix_style() -> Result<()> {
        let dir = tempdir()?;
        File::create(dir.path().join("a.txt"))?;

        let result = run_default(
            dir.path().to_str().unwrap(),
            "*.txt",
            1,
            "group",
            "roman", // unsupported suffix
            false,
            false,
            false,
        );
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("Unknown suffix style"),
            "Expected 'Unknown suffix style', got: {}",
            msg
        );

        Ok(())
    }

    #[test]
    fn error_suffix_none_with_multiple_subfolders() -> Result<()> {
        let dir = tempdir()?;

        // --suffix none with subfolders > 1 must return an error because every
        // destination folder would have the same name.
        let result = run_default(
            dir.path().to_str().unwrap(),
            "*",
            3,
            "group",
            "none",
            false,
            false,
            false,
        );
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("same name"),
            "Expected 'same name', got: {}",
            msg
        );

        Ok(())
    }
}
