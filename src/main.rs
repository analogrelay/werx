use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use forge::init::initialize_forge;
use forge::path::resolve_forge_path;

/// Forge - Manage your code repositories and workspaces
#[derive(Parser)]
#[command(name = "forge")]
#[command(about = "Manage your code repositories and workspaces", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Forge
    #[command(about = "Initialize a new Forge at the specified location")]
    Init {
        /// Path where the Forge should be created (defaults to ~/forge or $FORGE_DIR)
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,

        /// Force re-initialization of an existing Forge
        #[arg(short, long)]
        force: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path, force } => {
            cmd_init(path, force)?;
        }
    }

    Ok(())
}

fn cmd_init(cli_path: Option<PathBuf>, force: bool) -> Result<()> {
    // Resolve the target path
    let path = resolve_forge_path(cli_path)?;

    println!("Initializing Forge at: {}", path.display());

    // Initialize the Forge
    let forge = initialize_forge(path, force)?;

    // Success message
    println!();
    println!("✓ Forge initialized successfully!");
    println!();
    println!("Location: {}", forge.root.display());
    println!();
    println!("Next steps:");
    println!("  • Run 'forge add <repo-url>' to add a repository");
    println!("  • Run 'forge list' to see your repositories");
    println!();

    Ok(())
}
