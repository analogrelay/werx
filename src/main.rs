use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use forge::init::initialize_forge;
use forge::path::resolve_forge_path;
use forge::repos::{add_repo, list_repos, remove_repo};
use forge::Forge;

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

        /// Protocol preference for Git operations (ssh or https)
        #[arg(long, value_name = "PROTOCOL")]
        protocol: Option<String>,
    },

    /// Add a repository to the Forge (alias for 'repos add')
    #[command(about = "Add a repository to the Forge")]
    Add {
        /// Repository specification (URL, provider:owner/repo, or owner/repo)
        #[arg(value_name = "REPO")]
        repo: String,
    },

    /// Manage repositories in the Forge
    #[command(about = "Manage repositories in the Forge", subcommand)]
    Repos(ReposCommands),
}

#[derive(Subcommand)]
enum ReposCommands {
    /// Add a repository to the Forge
    #[command(about = "Add a repository to the Forge")]
    Add {
        /// Repository specification (URL, provider:owner/repo, or owner/repo)
        #[arg(value_name = "REPO")]
        repo: String,
    },

    /// List repositories in the Forge
    #[command(about = "List all repositories in the Forge")]
    List {
        /// Output format (text or json)
        #[arg(long, value_name = "FORMAT", default_value = "text")]
        format: String,
    },

    /// Remove a repository from the Forge
    #[command(about = "Remove a repository from the Forge")]
    Remove {
        /// Repository specification (URL, provider:owner/repo, or owner/repo)
        #[arg(value_name = "REPO")]
        repo: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path, force, protocol } => {
            cmd_init(path, force, protocol)?;
        }
        Commands::Add { repo } => {
            cmd_add(repo)?;
        }
        Commands::Repos(subcmd) => match subcmd {
            ReposCommands::Add { repo } => {
                cmd_add(repo)?;
            }
            ReposCommands::List { format } => {
                cmd_list(format)?;
            }
            ReposCommands::Remove { repo, force } => {
                cmd_remove(repo, force)?;
            }
        },
    }

    Ok(())
}

fn cmd_init(cli_path: Option<PathBuf>, force: bool, protocol_str: Option<String>) -> Result<()> {
    // Resolve the target path
    let path = resolve_forge_path(cli_path)?;

    println!("Initializing Forge at: {}", path.display());

    // Parse protocol if provided
    let protocol = if let Some(p) = protocol_str {
        Some(p.parse()?)
    } else {
        None
    };

    // Initialize the Forge
    let forge = initialize_forge(path, force, protocol)?;

    // Load config to show protocol preference
    let config = forge.load_config()?;

    // Success message
    println!();
    println!("✓ Forge initialized successfully!");
    println!();
    println!("Location: {}", forge.root.display());
    if let Some(prot) = config.protocol() {
        println!("Protocol: {}", prot);
    }
    println!();
    println!("Next steps:");
    println!("  • Run 'forge add <repo-url>' to add a repository");
    println!("  • Run 'forge repos list' to see your repositories");
    println!();

    Ok(())
}

fn cmd_add(repo: String) -> Result<()> {
    // Find the Forge
    let forge = find_forge()?;

    // Add the repository
    add_repo(&forge, &repo)?;

    Ok(())
}

fn cmd_list(format: String) -> Result<()> {
    // Find the Forge
    let forge = find_forge()?;

    // List repositories
    let repos = list_repos(&forge)?;

    if repos.is_empty() {
        println!("No repositories found.");
        println!();
        println!("Run 'forge add <repo>' to add a repository.");
        return Ok(());
    }

    match format.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&repos)?;
            println!("{}", json);
        }
        "text" | _ => {
            println!();
            println!("Repositories in Forge:");
            println!();

            for repo in &repos {
                if repo.valid {
                    println!("  • {}", repo.dir_name);
                    println!("    URL:    {}", repo.clone_url);
                    if let Some(branch) = &repo.default_branch {
                        println!("    Branch: {}", branch);
                    }
                    println!();
                } else {
                    println!("  • {} [INVALID]", repo.dir_name);
                    if let Some(error) = &repo.error {
                        println!("    Error: {}", error);
                    }
                    println!();
                }
            }

            println!("Total: {} repositories", repos.len());
            println!();
        }
    }

    Ok(())
}

fn cmd_remove(repo: String, force: bool) -> Result<()> {
    // Find the Forge
    let forge = find_forge()?;

    // Remove the repository
    remove_repo(&forge, &repo, force)?;

    Ok(())
}

/// Find the Forge at the default location
fn find_forge() -> Result<Forge> {
    let path = resolve_forge_path(None)?;

    if !Forge::exists_at(&path) {
        return Err(anyhow::anyhow!(
            "No Forge found at '{}'. Run 'forge init' first.",
            path.display()
        ));
    }

    Ok(Forge { root: path })
}
