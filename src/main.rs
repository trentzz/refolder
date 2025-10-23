use clap::Parser;


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
}


fn main() -> anyhow::Result<()> {
let args = Args::parse();
if args.subfolders == 0 {
anyhow::bail!("--subfolders must be greater than zero");
}
refolder::run(
&args.path,
&args.matching,
args.subfolders,
&args.prefix,
&args.suffix,
args.recursive,
args.dry_run,
args.force,
)
}