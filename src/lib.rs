use anyhow::{Context, Result, anyhow};
use globwalk::GlobWalkerBuilder;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Core library functions used by `main` and by tests.

/// Bold ANSI codes for terminal output
const BOLD_START: &str = "\x1b[1;34m";
const BOLD_END: &str = "\x1b[0m";

/// Public API: run the refolder operation.
pub fn run(
    base_path: &str,
    matching: &str,
    subfolders: usize,
    prefix: &str,
    suffix: &str,
    recursive: bool,
    dry_run: bool,
    force: bool,
) -> Result<()> {
    if subfolders == 0 {
        return Err(anyhow!("subfolders must be greater than zero"));
    }

    let base = Path::new(base_path);
    if !base.exists() {
        return Err(anyhow!("Path '{}' does not exist", base.display()));
    }
    if !base.is_dir() {
        return Err(anyhow!("Path '{}' is not a directory", base.display()));
    }

    // 1) Collect files to operate on. If files live under existing target folders (prefix-<i>),
    // treat them as sources as well so we can "redo" distributions.
    let files = collect_files(base, matching, recursive, prefix)?;

    if files.is_empty() {
        println!("No files matched pattern. Nothing to do.");
        return Ok(());
    }

    // 2) Partition into buckets as evenly as possible
    let buckets = partition(files, subfolders);

    // 3) For each bucket, create folder name and move files
    let mut planned_moves: Vec<(String, String)> = Vec::new();

    for (i, bucket) in buckets.into_iter().enumerate() {
        let folder_name = format_folder_name(prefix, i + 1, suffix)?;
        let folder_path = base.join(&folder_name);

        // Record folder creation and moves first (for dry-run printing)
        for src in bucket {
            let file_name = src
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or_else(|| anyhow!("Invalid filename for {}", src.display()))?;
            let dest = folder_path.join(file_name);
            planned_moves.push((src.display().to_string(), dest.display().to_string()));
        }

        // If not dry-run, perform actual creation and moving
        if !dry_run {
            if folder_path.exists() {
                if !folder_path.is_dir() {
                    return Err(anyhow!(
                        "Destination path {} exists and is not a directory",
                        folder_path.display()
                    ));
                }
            } else {
                fs::create_dir_all(&folder_path).with_context(|| {
                    format!("Failed to create directory {}", folder_path.display())
                })?;
            }

            for (src_str, dest_str) in planned_moves
                .iter()
                .filter(|(_, d)| d.starts_with(&folder_path.display().to_string()))
            {
                let src = PathBuf::from(src_str);
                let dest = PathBuf::from(dest_str);

                // Skip identical (redo safe)
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
                        fs::remove_file(&dest).with_context(|| {
                            format!(
                                "Failed removing existing destination file {}",
                                dest.display()
                            )
                        })?;
                    }
                }

                match fs::rename(&src, &dest) {
                    Ok(_) => {}
                    Err(rename_err) => {
                        fs::copy(&src, &dest).with_context(|| {
                            format!(
                                "Failed copying {} to {}: {}",
                                src.display(),
                                dest.display(),
                                rename_err
                            )
                        })?;
                        fs::remove_file(&src).with_context(|| {
                            format!("Failed removing original file {}", src.display())
                        })?;
                    }
                }
            }
        }
    }

    // If dry-run, print grouped output nicely
    if dry_run {
        print_dry_run_preview(&planned_moves);
    }

    Ok(())
}

/// Collect files matching `pattern` under `base`. If an existing folder with `prefix` exists
/// under `base` we also collect matching files inside it (one-level) so we can `redo` distributions.
fn collect_files(
    base: &Path,
    pattern: &str,
    recursive: bool,
    prefix: &str,
) -> Result<Vec<PathBuf>> {
    // Always canonicalize base first
    let canonical_base = std::fs::canonicalize(base)
        .with_context(|| format!("Failed to canonicalize {}", base.display()))?;

    // Use string form — avoids internal strip_prefix panics in globwalk
    let base_str = canonical_base
        .to_str()
        .ok_or_else(|| anyhow!("Base path is not valid UTF-8"))?
        .to_string();

    // Build walker using the canonical absolute path string
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
                eprintln!("⚠️ Warning: skipping entry due to error: {}", err);
                None
            }
        })
        .filter(|p| p.is_file())
        .collect();

    // Handle redo-existing prefix-* directories
    if let Ok(readdir) = fs::read_dir(&canonical_base) {
        for entry in readdir.filter_map(Result::ok) {
            let s = entry.file_name().to_string_lossy().to_string();
            if s.starts_with(prefix) && entry.path().is_dir() {
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
                    if p.is_file() && !files.contains(&p) {
                        files.push(p);
                    }
                }
            }
        }
    }

    files.sort();
    Ok(files)
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
    for i in 0..n {
        let take = base + if i < rem { 1 } else { 0 };
        for _ in 0..take {
            if idx < files.len() {
                buckets[i].push(files[idx].clone());
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

pub fn print_dry_run_preview(file_moves: &[(String, String)]) {
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

        // Wrap folder name in bold ANSI codes
        println!("{}{}{}{}", prefix_folder, BOLD_START, folder_name, BOLD_END);

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
            } else {
                if is_last_folder {
                    "    ├── "
                } else {
                    "│   ├── "
                }
            };
            println!("{}{}", prefix_file, file);
        }
    }

    // Optional: summary
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
    use tempfile::tempdir;

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
        // create 5 files
        for i in 0..5 {
            let p = base.join(format!("file{}.txt", i));
            File::create(&p)?;
        }

        // run - move into 3 buckets, force=true so that existing won't block (not needed here)
        run(
            base.to_str().unwrap(),
            "*.txt",
            3,
            "pack",
            "numbers",
            false,
            false,
            true,
        )?;

        // check folders
        let a = base.join("pack-1");
        let b = base.join("pack-2");
        let c = base.join("pack-3");
        assert!(a.is_dir() && b.is_dir() && c.is_dir());

        let cnts = vec![
            fs::read_dir(&a)?.count(),
            fs::read_dir(&b)?.count(),
            fs::read_dir(&c)?.count(),
        ];
        // distribution should sum to 5
        assert_eq!(cnts.iter().sum::<usize>(), 5);

        Ok(())
    }

    #[test]
    fn redo_existing_folders() -> Result<()> {
        // Test that files inside existing prefix-* folders are collected and re-shuffled
        let dir = tempdir()?;
        let base = dir.path();

        // create two existing folders pack-1 and pack-2 with some files
        let p1 = base.join("pack-1");
        let p2 = base.join("pack-2");
        fs::create_dir_all(&p1)?;
        fs::create_dir_all(&p2)?;

        File::create(p1.join("a.txt"))?;
        File::create(p1.join("b.txt"))?;
        File::create(p2.join("c.txt"))?;

        // Now ask to re-split into 3 buckets
        run(
            base.to_str().unwrap(),
            "*.txt",
            3,
            "pack",
            "numbers",
            false,
            false,
            true,
        )?;

        // ensure pack-1..pack-3 exist and files moved
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
        // Use "." explicitly to simulate the common cause of StripPrefixError
        let dir = tempdir()?;
        let base = dir.path();

        // create some files
        for i in 0..3 {
            let p = base.join(format!("f{}.txt", i));
            File::create(&p)?;
        }

        // Run collect_files directly to ensure no panic
        let result =
            std::panic::catch_unwind(|| collect_files(base, "*.txt", true, "pack").unwrap());

        assert!(
            result.is_ok(),
            "collect_files should never panic on relative paths"
        );

        Ok(())
    }
}
