use anyhow::Result;
use clap::Parser;
use compose_builder::collect_files;

/// Generate compose2nix modules from compose files
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Root directory that contains all compose files
    #[arg(default_value = "compose")]
    directory: String,

    /// Name of the project for compose2nix
    #[arg(short, long)]
    project: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let files = collect_files(&args.directory)?;
    for file in files {
        dbg!(file);
    }

    Ok(())
}
