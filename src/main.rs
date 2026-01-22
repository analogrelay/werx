use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use forge::Forge;
use forge::init::initialize_forge;
use forge::path::resolve_forge_path;
use forge::repos::{add_repo, list_repos, remove_repo};
use forge::workspace::{
    check_workspace_status, confirm_workspace_removal, create_worktree, detect_current_workspace,
    find_repository, list_workspaces, prompt_workspace_name, remove_workspace, select_repository,
};

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

    /// Manage workspaces in the Forge
    #[command(about = "Manage workspaces in the Forge", subcommand)]
    Workspace(WorkspaceCommands),

    /// Manage workspaces in the Forge (alias for 'workspace')
    #[command(about = "Manage workspaces in the Forge", subcommand, alias = "wt")]
    Workspaces(WorkspaceCommands),
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

#[derive(Subcommand)]
enum WorkspaceCommands {
    /// Create a new workspace
    #[command(about = "Create a new workspace from a repository")]
    Create {
        /// Repository specification (optional if running from within a workspace)
        #[arg(value_name = "REPO")]
        repo: Option<String>,

        /// Branch name (defaults to repository's default branch)
        #[arg(value_name = "BRANCH")]
        branch: Option<String>,

        /// Custom workspace name (defaults to branch name)
        #[arg(long, short, value_name = "NAME")]
        name: Option<String>,
    },

    /// List all workspaces in the Forge
    #[command(about = "List all workspaces in the Forge")]
    List {
        /// Output format (text or json)
        #[arg(long, value_name = "FORMAT", default_value = "text")]
        format: String,
    },

    /// Remove a workspace
    #[command(about = "Remove a workspace", alias = "rm")]
    Remove {
        /// Workspace path (repo/workspace or just workspace)
        #[arg(value_name = "WORKSPACE")]
        workspace: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init {
            path,
            force,
            protocol,
        } => {
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
        Commands::Workspace(subcmd) | Commands::Workspaces(subcmd) => match subcmd {
            WorkspaceCommands::Create { repo, branch, name } => {
                cmd_workspace_create(repo, branch, name)?;
            }
            WorkspaceCommands::List { format } => {
                cmd_workspace_list(format)?;
            }
            WorkspaceCommands::Remove { workspace, force } => {
                cmd_workspace_remove(workspace, force)?;
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

fn cmd_workspace_create(
    repo_spec: Option<String>,
    branch: Option<String>,
    name: Option<String>,
) -> Result<()> {
    // Find the Forge
    let forge = find_forge()?;

    // Resolve repository
    let repo_info = if let Some(spec) = repo_spec {
        // Repository specified explicitly
        find_repository(&forge, &spec)?
    } else {
        // Try to detect current workspace
        let current_dir = std::env::current_dir()?;
        if let Some(repo) = detect_current_workspace(&current_dir, &forge)? {
            println!();
            println!(
                "Using repository from current workspace: {}",
                repo.clone_url
            );
            repo
        } else {
            // Interactive selector
            select_repository(&forge)?
        }
    };

    // Determine branch
    let branch_name = if let Some(b) = branch {
        b
    } else {
        // Use repository's default branch
        repo_info.default_branch.clone().ok_or_else(|| {
            anyhow::anyhow!(
                "Could not determine default branch for repository.\n\
                 Please specify a branch explicitly: forge workspace create <repo> <branch>"
            )
        })?
    };

    // Determine workspace name
    let workspace_name = if let Some(n) = name {
        n
    } else {
        // Prompt for workspace name with branch name as default
        prompt_workspace_name(&branch_name)?
    };

    println!();
    println!("Creating workspace...");

    // Create the worktree
    let workspace_path = create_worktree(&forge, &repo_info, &workspace_name, &branch_name)?;

    println!();
    println!("✓ Workspace created successfully!");
    println!();
    println!("  Repository: {}", repo_info.clone_url);
    println!("  Branch:     {}", branch_name);
    println!("  Workspace:  {}/{}", repo_info.dir_name, workspace_name);
    println!("  Path:       {}", workspace_path.display());
    println!();
    println!("Next steps:");
    println!("  cd {}", workspace_path.display());
    println!();

    Ok(())
}

fn cmd_workspace_list(format: String) -> Result<()> {
    // Find the Forge
    let forge = find_forge()?;

    // List workspaces
    let workspaces = list_workspaces(&forge)?;

    if workspaces.is_empty() {
        println!("No workspaces found.");
        println!();
        println!("Run 'forge workspace create' to create a workspace.");
        return Ok(());
    }

    match format.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&workspaces)?;
            println!("{}", json);
        }
        "text" | _ => {
            println!();
            println!("Workspaces in Forge:");
            println!();

            for workspace in &workspaces {
                println!("  • {}/{}", workspace.repository, workspace.name);
                println!("    Path:   {}", workspace.path.display());
                if let Some(branch) = &workspace.branch {
                    println!("    Branch: {}", branch);
                }

                // Show status if not clean
                match workspace.status {
                    forge::WorkspaceStatus::Clean => {}
                    forge::WorkspaceStatus::Modified => println!("    Status: Modified"),
                    forge::WorkspaceStatus::Untracked => println!("    Status: Untracked files"),
                    forge::WorkspaceStatus::Locked => println!("    Status: Locked"),
                    forge::WorkspaceStatus::Prunable => {
                        println!("    Status: Prunable (directory missing)")
                    }
                }

                println!();
            }

            println!("Total: {} workspaces", workspaces.len());
            println!();
        }
    }

    Ok(())
}

fn cmd_workspace_remove(workspace: String, force: bool) -> Result<()> {
    // Find the Forge
    let forge = find_forge()?;

    // Get all workspaces to find the one to remove
    let workspaces = list_workspaces(&forge)?;

    // Find the workspace
    let matching_workspaces: Vec<&forge::Workspace> = workspaces
        .iter()
        .filter(|w| {
            let full_name = format!("{}/{}", w.repository, w.name);
            full_name == workspace || w.name == workspace
        })
        .collect();

    if matching_workspaces.is_empty() {
        return Err(anyhow::anyhow!(
            "Workspace not found: {}\n\n\
             Run 'forge workspace list' to see available workspaces.",
            workspace
        ));
    }

    if matching_workspaces.len() > 1 {
        println!();
        println!(
            "Multiple workspaces match '{}'. Please specify the full path:",
            workspace
        );
        println!();
        for ws in &matching_workspaces {
            println!("  {}/{}", ws.repository, ws.name);
        }
        println!();
        return Err(anyhow::anyhow!("Ambiguous workspace name"));
    }

    let ws = matching_workspaces[0];

    // Check workspace status
    let status = check_workspace_status(&ws.path)?;

    // Confirm removal
    let confirmed = confirm_workspace_removal(&ws.name, &ws.path, &status, force)?;

    if !confirmed {
        println!();
        println!("Operation cancelled.");
        return Ok(());
    }

    // Remove the workspace
    remove_workspace(&forge, &workspace)?;

    println!();
    println!("✓ Workspace removed successfully!");
    println!();
    println!("  Workspace: {}/{}", ws.repository, ws.name);
    println!();

    Ok(())
}
