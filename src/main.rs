use clap::Parser;

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

fn main() {
    let args = Args::parse();

    println!("{:?}", args);
}
