use clap::Parser;
use std::io::IsTerminal;

/// Move matching files into equally-sized subfolders
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the directory to search
    path: String,

    /// Glob pattern for matching files (shell-style). Default: "*"
    #[arg(short, long, default_value = "*")]
    matching: String,

    /// Number of subfolders to split into
    #[arg(short, long)]
    subfolders: usize,

    /// Prefix for created subfolders. Default: "group"
    #[arg(short, long, default_value = "group")]
    prefix: String,

    /// Suffix style: numbers | letters | none
    #[arg(long, default_value = "numbers")]
    suffix: String,

    /// Recurse into subdirectories
    #[arg(short, long)]
    recursive: bool,

    /// Print actions without performing them
    #[arg(long)]
    dry_run: bool,

    /// Overwrite existing files/folders in destination
    #[arg(short, long)]
    force: bool,

    /// Suppress all ANSI colour codes in output
    #[arg(long)]
    no_color: bool,

    /// Print each file move to stderr as it happens
    #[arg(short, long)]
    verbose: bool,

    /// Sort order for files before distribution: name (default) | none | size | size-desc
    #[arg(long, default_value = "name")]
    sort: String,

    /// Directory where subfolders are created. Defaults to the source path.
    #[arg(short, long)]
    output_dir: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if args.subfolders == 0 {
        anyhow::bail!("--subfolders must be greater than zero");
    }

    // Colour is enabled when stdout is a TTY, --no-color is not set,
    // and the NO_COLOR environment variable is absent.
    let color =
        std::io::stdout().is_terminal() && !args.no_color && std::env::var("NO_COLOR").is_err();

    refolder::run(&refolder::Config {
        base_path: &args.path,
        matching: &args.matching,
        subfolders: args.subfolders,
        prefix: &args.prefix,
        suffix: &args.suffix,
        recursive: args.recursive,
        dry_run: args.dry_run,
        force: args.force,
        color,
        verbose: args.verbose,
        sort: &args.sort,
        output_dir: args.output_dir.as_deref(),
    })
}
