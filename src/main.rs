use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use werx::{
    AgentType, SpawnOptions, Werx, WorkspaceStatusDetails, add_repo, attach_to_agent,
    check_workspace_status, cmd_shell_init, confirm_workspace_removal, create_repo,
    create_worktree, detect_current_workspace, detect_providers, emit_change_directory, find_agent,
    find_repository, fuzzy_select_repository, get_default_provider, get_workspace_status_details,
    initialize_werx, kill_agent, list_agents, list_repos, list_workspaces, prompt_branch_name,
    prompt_workspace_name, remove_repo, remove_workspace, resolve_werx_path, select_repository,
    select_workspace_with_query, spawn_agent,
};

/// Werx - Manage your code repositories and workspaces
#[derive(Parser)]
#[command(name = "werx")]
#[command(about = "Manage your code repositories and workspaces", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Werx
    #[command(about = "Initialize a new Werx at the specified location")]
    Init {
        /// Path where the Werx should be created (defaults to ~/werx or $WERX_DIR)
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,

        /// Force re-initialization of an existing Werx
        #[arg(short, long)]
        force: bool,

        /// Protocol preference for Git operations (ssh or https)
        #[arg(long, value_name = "PROTOCOL")]
        protocol: Option<String>,
    },

    /// Add a repository to the Werx (alias for 'repos add')
    #[command(about = "Add a repository to the Werx")]
    Add {
        /// Repository specification (URL, provider:owner/repo, or owner/repo)
        #[arg(value_name = "REPO")]
        repo: String,
    },

    /// Create a new repository from scratch (alias for 'repos create')
    #[command(about = "Create a new repository from scratch")]
    Create {
        /// Repository specification in owner/repo format
        #[arg(value_name = "REPO")]
        repo: String,
    },

    /// Manage repositories in the Werx
    #[command(about = "Manage repositories in the Werx", subcommand)]
    Repos(ReposCommands),

    /// Manage workspaces in the Werx
    #[command(
        about = "Manage workspaces in the Werx",
        subcommand,
        alias = "wt",
        alias = "workspaces",
        alias = "workspace",
        alias = "worktree"
    )]
    Work(WorkspaceCommands),

    /// Navigate to a workspace using fuzzy search
    #[command(
        about = "Navigate to a workspace using fuzzy search",
        long_about = "Navigate to a workspace using fuzzy search.\n\n\
                      Examples:\n  \
                      werx go              # Launch interactive fuzzy search\n  \
                      werx go feature      # Pre-fill search with 'feature'\n  \
                      werx go repo/main    # Direct navigation if exact match\n\n\
                      Note: Requires shell integration. Run 'werx shell init --help' for setup."
    )]
    Go {
        /// Optional query to pre-fill or match workspaces
        #[arg(value_name = "QUERY")]
        query: Option<String>,
    },

    /// Shell integration commands
    #[command(
        about = "Shell integration commands",
        long_about = "Shell integration commands for enabling directory navigation.\n\n\
                      To enable 'werx go' to change your shell's directory, add to your shell config:\n\n  \
                      Bash: eval \"$(werx shell init bash)\"  (add to ~/.bashrc)\n  \
                      Zsh:  eval \"$(werx shell init zsh)\"   (add to ~/.zshrc)",
        subcommand
    )]
    Shell(ShellCommands),

    /// Manage AI coding agents
    #[command(about = "Manage AI coding agents", subcommand, alias = "agents")]
    Agent(AgentCommands),
}

#[derive(Subcommand)]
enum ShellCommands {
    /// Output shell initialization code
    #[command(
        about = "Output shell initialization code for the specified shell",
        long_about = "Output shell initialization code for the specified shell.\n\n\
                      Supported shells: bash, zsh\n\n\
                      Examples:\n  \
                      eval \"$(werx shell init bash)\"  # Add to ~/.bashrc\n  \
                      eval \"$(werx shell init zsh)\"   # Add to ~/.zshrc"
    )]
    Init {
        /// Shell type (bash or zsh)
        #[arg(value_name = "SHELL")]
        shell: String,
    },
}

#[derive(Subcommand)]
enum AgentCommands {
    /// Spawn a new coding agent
    #[command(
        about = "Spawn a new coding agent in an isolated worktree",
        long_about = "Spawn a new AI coding agent in an isolated git worktree.\n\n\
                      The agent runs in a tmux session for persistence and easy access.\n\n\
                      If no repository is specified, you'll be prompted to select one.\n\
                      If no branch is specified, you'll be prompted to create a new branch.\n\n\
                      Examples:\n  \
                      werx agent spawn                           # Interactive mode\n  \
                      werx agent spawn owner/repo                # Prompts for branch name\n  \
                      werx agent spawn owner/repo -b feature     # Use existing branch\n  \
                      werx agent spawn --agent claude            # Use Claude instead of OpenCode"
    )]
    Spawn {
        /// Repository specification (optional, prompts if not provided)
        #[arg(value_name = "REPO")]
        repo: Option<String>,

        /// Branch to checkout (prompts for new branch name if not specified)
        #[arg(short, long, value_name = "BRANCH")]
        branch: Option<String>,

        /// Agent to use (opencode, claude, copilot)
        #[arg(short, long, value_name = "AGENT")]
        agent: Option<String>,

        /// Initial prompt to send to the agent
        #[arg(short = 'P', long, value_name = "PROMPT")]
        prompt: Option<String>,

        /// Open editor to compose initial prompt
        #[arg(short = 'e', long = "edit-prompt")]
        edit_prompt: bool,
    },

    /// List running agents
    #[command(about = "List all running agents")]
    List {
        /// Output format (text or json)
        #[arg(long, value_name = "FORMAT", default_value = "text")]
        format: String,
    },

    /// Show detailed agent status
    #[command(about = "Show detailed status of agents")]
    Status {
        /// Agent name (optional, shows all if not specified)
        #[arg(value_name = "AGENT")]
        agent: Option<String>,

        /// Output format (text or json)
        #[arg(long, value_name = "FORMAT", default_value = "text")]
        format: String,
    },

    /// Attach to the agent tmux session
    #[command(about = "Attach to the agent tmux session")]
    Attach {
        /// Agent name (optional, attaches to session if not specified)
        #[arg(value_name = "AGENT")]
        agent: Option<String>,
    },

    /// Kill a running agent
    #[command(about = "Kill a running agent")]
    Kill {
        /// Agent name
        #[arg(value_name = "AGENT")]
        agent: Option<String>,

        /// Also remove the agent's worktree
        #[arg(long)]
        cleanup: bool,
    },

    /// List available agent providers
    #[command(about = "List available agent providers")]
    Providers,
}

#[derive(Subcommand)]
enum ReposCommands {
    /// Add a repository to the Werx
    #[command(about = "Add a repository to the Werx")]
    Add {
        /// Repository specification (URL, provider:owner/repo, or owner/repo)
        #[arg(value_name = "REPO")]
        repo: String,
    },

    /// Create a new repository from scratch
    #[command(about = "Create a new repository from scratch")]
    Create {
        /// Repository specification in owner/repo format
        #[arg(value_name = "REPO")]
        repo: String,
    },

    /// List repositories in the Werx
    #[command(about = "List all repositories in the Werx")]
    List {
        /// Output format (text or json)
        #[arg(long, value_name = "FORMAT", default_value = "text")]
        format: String,
    },

    /// Remove a repository from the Werx
    #[command(about = "Remove a repository from the Werx")]
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

    /// List all workspaces in the Werx
    #[command(about = "List all workspaces in the Werx")]
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

    /// Navigate to a workspace using fuzzy search (alias for 'werx go')
    #[command(
        about = "Navigate to a workspace using fuzzy search (alias for 'werx go')",
        long_about = "Navigate to a workspace using fuzzy search.\n\n\
                      This is an alias for 'werx go' that works from the workspace subcommand.\n\n\
                      Examples:\n  \
                      werx workspace go              # Launch interactive fuzzy search\n  \
                      werx workspace go feature      # Pre-fill search with 'feature'\n\n\
                      Note: Requires shell integration. Run 'werx shell init --help' for setup."
    )]
    Go {
        /// Optional query to pre-fill or match workspaces
        #[arg(value_name = "QUERY")]
        query: Option<String>,
    },

    /// Show comprehensive workspace status
    #[command(about = "Show workspace status across the Werx")]
    Status {
        /// Filter to a specific repository
        #[arg(value_name = "REPO")]
        repo: Option<String>,

        /// Output format (text or json)
        #[arg(long, value_name = "FORMAT", default_value = "text")]
        format: String,
    },

    /// Check workspaces for specific conditions
    #[command(about = "Check workspaces for specific conditions")]
    Check {
        /// Filter to a specific repository
        #[arg(value_name = "REPO")]
        repo: Option<String>,

        /// Check for uncommitted changes only
        #[arg(long)]
        uncommitted: bool,

        /// Check for unpushed branches only
        #[arg(long)]
        unpushed: bool,

        /// Check for merged branches only
        #[arg(long)]
        merged: bool,

        /// Output format (text or json)
        #[arg(long, value_name = "FORMAT", default_value = "text")]
        format: String,
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
        Commands::Create { repo } => {
            cmd_create(repo)?;
        }
        Commands::Go { query } => {
            cmd_go(query)?;
        }
        Commands::Shell(subcmd) => match subcmd {
            ShellCommands::Init { shell } => {
                cmd_shell_init(&shell)?;
            }
        },
        Commands::Repos(subcmd) => match subcmd {
            ReposCommands::Add { repo } => {
                cmd_add(repo)?;
            }
            ReposCommands::Create { repo } => {
                cmd_create(repo)?;
            }
            ReposCommands::List { format } => {
                cmd_list(format)?;
            }
            ReposCommands::Remove { repo, force } => {
                cmd_remove(repo, force)?;
            }
        },
        Commands::Work(subcmd) => match subcmd {
            WorkspaceCommands::Create { repo, branch, name } => {
                cmd_workspace_create(repo, branch, name)?;
            }
            WorkspaceCommands::List { format } => {
                cmd_workspace_list(format)?;
            }
            WorkspaceCommands::Remove { workspace, force } => {
                cmd_workspace_remove(workspace, force)?;
            }
            WorkspaceCommands::Go { query } => {
                cmd_go(query)?;
            }
            WorkspaceCommands::Status { repo, format } => {
                cmd_workspace_status(repo, format)?;
            }
            WorkspaceCommands::Check {
                repo,
                uncommitted,
                unpushed,
                merged,
                format,
            } => {
                cmd_workspace_check(repo, uncommitted, unpushed, merged, format)?;
            }
        },
        Commands::Agent(subcmd) => match subcmd {
            AgentCommands::Spawn {
                repo,
                branch,
                agent,
                prompt,
                edit_prompt,
            } => {
                cmd_agent_spawn(repo, branch, agent, prompt, edit_prompt)?;
            }
            AgentCommands::List { format } => {
                cmd_agent_list(format)?;
            }
            AgentCommands::Status { agent, format } => {
                cmd_agent_status(agent, format)?;
            }
            AgentCommands::Attach { agent } => {
                cmd_agent_attach(agent)?;
            }
            AgentCommands::Kill { agent, cleanup } => {
                cmd_agent_kill(agent, cleanup)?;
            }
            AgentCommands::Providers => {
                cmd_agent_providers()?;
            }
        },
    }

    Ok(())
}

fn cmd_init(cli_path: Option<PathBuf>, force: bool, protocol_str: Option<String>) -> Result<()> {
    // Resolve the target path
    let path = resolve_werx_path(cli_path)?;

    println!("Initializing Werx at: {}", path.display());

    // Parse protocol if provided
    let protocol = if let Some(p) = protocol_str {
        Some(p.parse()?)
    } else {
        None
    };

    // Initialize the Werx
    let werx = initialize_werx(path, force, protocol)?;

    // Load config to show protocol preference
    let config = werx.load_config()?;

    // Success message
    println!();
    println!("Werx initialized successfully!");
    println!();
    println!("Location: {}", werx.root.display());
    if let Some(prot) = config.protocol() {
        println!("Protocol: {}", prot);
    }
    println!();
    println!("Next steps:");
    println!("  - Run 'werx add <repo-url>' to add a repository");
    println!("  - Run 'werx repos list' to see your repositories");

    // Suggest shell integration based on user's shell
    if let Ok(shell_var) = std::env::var("SHELL") {
        let shell_name = std::path::Path::new(&shell_var)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        if shell_name == "bash" || shell_name == "zsh" {
            println!();
            println!("Shell integration (optional):");
            println!(
                "  - Enable navigation with 'werx go' by adding to your .{}rc:",
                shell_name
            );
            println!("    eval \"$(werx shell init {})\"", shell_name);
        }
    }

    println!();

    Ok(())
}

fn cmd_add(repo: String) -> Result<()> {
    // Find the Werx
    let werx = find_werx()?;

    // Add the repository
    add_repo(&werx, &repo)?;

    Ok(())
}

fn cmd_create(repo: String) -> Result<()> {
    // Find the Werx
    let werx = find_werx()?;

    // Create the repository
    let created_info = create_repo(&werx, &repo)?;

    // Now create the worktree on main
    println!("Creating workspace on main branch...");

    // Build RepoInfo for the newly created repository
    let repo_info = werx::RepoInfo {
        dir_name: created_info.dir_name.clone(),
        clone_url: format!(
            "https://github.com/{}/{}.git",
            created_info.owner, created_info.name
        ),
        normalized_url: format!(
            "https://github.com/{}/{}.git",
            created_info.owner.to_lowercase(),
            created_info.name.to_lowercase()
        ),
        default_branch: Some("main".to_string()),
        valid: true,
        error: None,
    };

    // Create worktree on main
    let workspace_path = create_worktree(&werx, &repo_info, "main", "main")?;

    println!();
    println!("Repository created successfully!");
    println!();
    println!("  Repository: {}/{}", created_info.owner, created_info.name);
    println!("  Location:   .werx/repos/{}", created_info.dir_name);
    println!("  Workspace:  {}/main", created_info.dir_name);
    println!("  Path:       {}", workspace_path.display());
    println!();
    println!("Next steps:");
    println!("  cd {}", workspace_path.display());
    println!();
    println!("When ready to publish:");
    println!("  Create the repository on GitHub/GitLab, then:");
    println!("  git push -u origin main");
    println!();

    Ok(())
}

fn cmd_list(format: String) -> Result<()> {
    // Find the Werx
    let werx = find_werx()?;

    // List repositories
    let repos = list_repos(&werx)?;

    if repos.is_empty() {
        println!("No repositories found.");
        println!();
        println!("Run 'werx add <repo>' to add a repository.");
        return Ok(());
    }

    match format.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&repos)?;
            println!("{}", json);
        }
        _ => {
            println!();
            println!("Repositories in Werx:");
            println!();

            for repo in &repos {
                if repo.valid {
                    println!("  - {}", repo.dir_name);
                    println!("    URL:    {}", repo.clone_url);
                    if let Some(branch) = &repo.default_branch {
                        println!("    Branch: {}", branch);
                    }
                    println!();
                } else {
                    println!("  - {} [INVALID]", repo.dir_name);
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
    // Find the Werx
    let werx = find_werx()?;

    // Remove the repository
    remove_repo(&werx, &repo, force)?;

    Ok(())
}

/// Find the Werx at the default location
fn find_werx() -> Result<Werx> {
    let path = resolve_werx_path(None)?;

    if !Werx::exists_at(&path) {
        return Err(anyhow::anyhow!(
            "No Werx found at '{}'. Run 'werx init' first.",
            path.display()
        ));
    }

    Ok(Werx { root: path })
}

fn cmd_workspace_create(
    repo_spec: Option<String>,
    branch: Option<String>,
    name: Option<String>,
) -> Result<()> {
    // Find the Werx
    let werx = find_werx()?;

    // Resolve repository
    let repo_info = if let Some(spec) = repo_spec {
        // Repository specified explicitly
        find_repository(&werx, &spec)?
    } else {
        // Try to detect current workspace
        let current_dir = std::env::current_dir()?;
        if let Some(repo) = detect_current_workspace(&current_dir, &werx)? {
            println!();
            println!(
                "Using repository from current workspace: {}",
                repo.clone_url
            );
            repo
        } else {
            // Interactive selector
            select_repository(&werx)?
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
                 Please specify a branch explicitly: werx workspace create <repo> <branch>"
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
    let workspace_path = create_worktree(&werx, &repo_info, &workspace_name, &branch_name)?;

    println!();
    println!("Workspace created successfully!");
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
    // Find the Werx
    let werx = find_werx()?;

    // List workspaces
    let workspaces = list_workspaces(&werx)?;

    if workspaces.is_empty() {
        println!("No workspaces found.");
        println!();
        println!("Run 'werx workspace create' to create a workspace.");
        return Ok(());
    }

    match format.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&workspaces)?;
            println!("{}", json);
        }
        _ => {
            println!();
            println!("Workspaces in Werx:");
            println!();

            for workspace in &workspaces {
                println!("  - {}/{}", workspace.repository, workspace.name);
                println!("    Path:   {}", workspace.path.display());
                if let Some(branch) = &workspace.branch {
                    println!("    Branch: {}", branch);
                }

                // Show status if not clean
                match workspace.status {
                    werx::WorkspaceStatus::Clean => {}
                    werx::WorkspaceStatus::Modified => println!("    Status: Modified"),
                    werx::WorkspaceStatus::Untracked => println!("    Status: Untracked files"),
                    werx::WorkspaceStatus::Locked => println!("    Status: Locked"),
                    werx::WorkspaceStatus::Prunable => {
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
    // Find the Werx
    let werx = find_werx()?;

    // Get all workspaces to find the one to remove
    let workspaces = list_workspaces(&werx)?;

    // Find the workspace
    let matching_workspaces: Vec<&werx::Workspace> = workspaces
        .iter()
        .filter(|w| {
            let full_name = format!("{}/{}", w.repository, w.name);
            full_name == workspace || w.name == workspace
        })
        .collect();

    if matching_workspaces.is_empty() {
        return Err(anyhow::anyhow!(
            "Workspace not found: {}\n\n\
             Run 'werx workspace list' to see available workspaces.",
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
    remove_workspace(&werx, &workspace)?;

    println!();
    println!("Workspace removed successfully!");
    println!();
    println!("  Workspace: {}/{}", ws.repository, ws.name);
    println!();

    Ok(())
}

fn cmd_go(query: Option<String>) -> Result<()> {
    // Find the Werx
    let werx = find_werx()?;

    // List all workspaces
    let workspaces = list_workspaces(&werx)?;

    if workspaces.is_empty() {
        println!("No workspaces found.");
        println!();
        println!("Run 'werx workspace create' to create a workspace.");
        return Ok(());
    }

    // Select workspace with optional query
    match select_workspace_with_query(workspaces, query)? {
        Some(workspace) => {
            // Emit change_directory directive
            emit_change_directory(&workspace.path);
        }
        None => {
            // User cancelled or no selection
            // Just exit without error
        }
    }

    Ok(())
}

/// Aggregated workspace status for display
#[derive(Debug)]
struct WorkspaceWithStatus {
    workspace: werx::Workspace,
    details: WorkspaceStatusDetails,
}

/// Summary of workspace status counts
#[derive(Debug, serde::Serialize)]
struct StatusSummary {
    total: usize,
    uncommitted: usize,
    unpushed: usize,
    merged: usize,
    clean: usize,
}

fn cmd_workspace_status(repo: Option<String>, format: String) -> Result<()> {
    // Find the Werx
    let werx = find_werx()?;

    // List workspaces (optionally filtered by repository)
    let mut workspaces = list_workspaces(&werx)?;

    // Filter by repository if specified
    if let Some(ref repo_spec) = repo {
        let repo_info = find_repository(&werx, repo_spec)?;
        workspaces.retain(|w| w.repository == repo_info.dir_name);
    }

    if workspaces.is_empty() {
        if let Some(ref repo_spec) = repo {
            println!("No workspaces found for repository '{}'.", repo_spec);
            println!();
            println!(
                "Run 'werx workspace create {}' to create a workspace.",
                repo_spec
            );
        } else {
            println!("No workspaces found.");
            println!();
            println!("Run 'werx workspace create' to create a workspace.");
        }
        return Ok(());
    }

    // Gather status for all workspaces
    let mut workspace_statuses: Vec<WorkspaceWithStatus> = Vec::new();
    for workspace in workspaces {
        let details = get_workspace_status_details(&workspace, &werx)?;
        workspace_statuses.push(WorkspaceWithStatus { workspace, details });
    }

    // Calculate summary
    let summary = StatusSummary {
        total: workspace_statuses.len(),
        uncommitted: workspace_statuses
            .iter()
            .filter(|w| w.details.uncommitted_changes)
            .count(),
        unpushed: workspace_statuses
            .iter()
            .filter(|w| w.details.unpushed_branch)
            .count(),
        merged: workspace_statuses
            .iter()
            .filter(|w| w.details.merged_branch)
            .count(),
        clean: workspace_statuses
            .iter()
            .filter(|w| {
                !w.details.uncommitted_changes
                    && !w.details.unpushed_branch
                    && !w.details.merged_branch
            })
            .count(),
    };

    match format.as_str() {
        "json" => {
            print_status_json(&workspace_statuses, &summary)?;
        }
        _ => {
            print_status_text(&workspace_statuses, &summary, repo.as_deref())?;
        }
    }

    Ok(())
}

fn print_status_text(
    statuses: &[WorkspaceWithStatus],
    summary: &StatusSummary,
    repo_filter: Option<&str>,
) -> Result<()> {
    println!();
    if let Some(repo) = repo_filter {
        println!("Workspace Status for '{}'", repo);
    } else {
        println!("Workspace Status for Werx");
    }
    println!();

    // Uncommitted changes section
    let uncommitted: Vec<_> = statuses
        .iter()
        .filter(|w| w.details.uncommitted_changes)
        .collect();
    if !uncommitted.is_empty() {
        println!(
            "Uncommitted Changes ({} workspace{}):",
            uncommitted.len(),
            if uncommitted.len() == 1 { "" } else { "s" }
        );
        for ws in &uncommitted {
            let change_summary = if let Some(ref details) = ws.details.status_details {
                let mut parts = Vec::new();
                if !details.modified_files.is_empty() {
                    parts.push(format!("M:{}", details.modified_files.len()));
                }
                if !details.untracked_files.is_empty() {
                    parts.push(format!("?:{}", details.untracked_files.len()));
                }
                parts.join(" ")
            } else {
                String::new()
            };
            println!(
                "  {}/{}  {}",
                ws.workspace.repository, ws.workspace.name, change_summary
            );
        }
        println!();
    }

    // Unpushed branches section
    let unpushed: Vec<_> = statuses
        .iter()
        .filter(|w| w.details.unpushed_branch)
        .collect();
    if !unpushed.is_empty() {
        println!(
            "Unpushed Branches ({} workspace{}):",
            unpushed.len(),
            if unpushed.len() == 1 { "" } else { "s" }
        );
        for ws in &unpushed {
            let branch = ws.details.branch_name.as_deref().unwrap_or("(unknown)");
            println!(
                "  {}/{}  Branch '{}' not on remote",
                ws.workspace.repository, ws.workspace.name, branch
            );
        }
        println!();
    }

    // Merged branches section
    let merged: Vec<_> = statuses
        .iter()
        .filter(|w| w.details.merged_branch)
        .collect();
    if !merged.is_empty() {
        println!(
            "Merged Branches ({} workspace{}):",
            merged.len(),
            if merged.len() == 1 { "" } else { "s" }
        );
        for ws in &merged {
            let branch = ws.details.branch_name.as_deref().unwrap_or("(unknown)");
            let default = ws.details.default_branch.as_deref().unwrap_or("main");
            println!(
                "  {}/{}  Branch '{}' merged to '{}'",
                ws.workspace.repository, ws.workspace.name, branch, default
            );
        }
        println!();
    }

    // Clean workspaces section
    let clean: Vec<_> = statuses
        .iter()
        .filter(|w| {
            !w.details.uncommitted_changes && !w.details.unpushed_branch && !w.details.merged_branch
        })
        .collect();
    if !clean.is_empty() {
        println!(
            "Clean Workspaces ({} workspace{}):",
            clean.len(),
            if clean.len() == 1 { "" } else { "s" }
        );
        for ws in &clean {
            println!("  {}/{}", ws.workspace.repository, ws.workspace.name);
        }
        println!();
    }

    // Summary
    println!(
        "Summary: {} total, {} uncommitted, {} unpushed, {} merged, {} clean",
        summary.total, summary.uncommitted, summary.unpushed, summary.merged, summary.clean
    );
    println!();

    Ok(())
}

fn print_status_json(statuses: &[WorkspaceWithStatus], summary: &StatusSummary) -> Result<()> {
    #[derive(serde::Serialize)]
    struct JsonOutput {
        workspaces: Vec<WorkspaceStatusJson>,
        summary: StatusSummary,
    }

    #[derive(serde::Serialize)]
    struct WorkspaceStatusJson {
        name: String,
        path: String,
        repository: String,
        branch: Option<String>,
        uncommitted_changes: bool,
        unpushed_branch: bool,
        merged_branch: bool,
        status_details: Option<StatusDetailsJson>,
    }

    #[derive(serde::Serialize)]
    struct StatusDetailsJson {
        modified_files: Vec<String>,
        untracked_files: Vec<String>,
    }

    let workspaces: Vec<WorkspaceStatusJson> = statuses
        .iter()
        .map(|ws| WorkspaceStatusJson {
            name: ws.workspace.name.clone(),
            path: ws.workspace.path.display().to_string(),
            repository: ws.workspace.repository.clone(),
            branch: ws.details.branch_name.clone(),
            uncommitted_changes: ws.details.uncommitted_changes,
            unpushed_branch: ws.details.unpushed_branch,
            merged_branch: ws.details.merged_branch,
            status_details: ws
                .details
                .status_details
                .as_ref()
                .map(|d| StatusDetailsJson {
                    modified_files: d.modified_files.clone(),
                    untracked_files: d.untracked_files.clone(),
                }),
        })
        .collect();

    let output = JsonOutput {
        workspaces,
        summary: StatusSummary {
            total: summary.total,
            uncommitted: summary.uncommitted,
            unpushed: summary.unpushed,
            merged: summary.merged,
            clean: summary.clean,
        },
    };

    let json = serde_json::to_string_pretty(&output)?;
    println!("{}", json);

    Ok(())
}

fn cmd_workspace_check(
    repo: Option<String>,
    uncommitted: bool,
    unpushed: bool,
    merged: bool,
    format: String,
) -> Result<()> {
    // Find the Werx
    let werx = find_werx()?;

    // List workspaces (optionally filtered by repository)
    let mut workspaces = list_workspaces(&werx)?;

    // Filter by repository if specified
    if let Some(ref repo_spec) = repo {
        let repo_info = find_repository(&werx, repo_spec)?;
        workspaces.retain(|w| w.repository == repo_info.dir_name);
    }

    if workspaces.is_empty() {
        if let Some(ref repo_spec) = repo {
            println!("No workspaces found for repository '{}'.", repo_spec);
            println!();
            println!(
                "Run 'werx workspace create {}' to create a workspace.",
                repo_spec
            );
        } else {
            println!("No workspaces found.");
            println!();
            println!("Run 'werx workspace create' to create a workspace.");
        }
        return Ok(());
    }

    // Determine which checks to perform (default: all if none specified)
    let all_checks = !uncommitted && !unpushed && !merged;
    let check_uncommitted = uncommitted || all_checks;
    let check_unpushed = unpushed || all_checks;
    let check_merged = merged || all_checks;

    // Gather status for all workspaces
    let mut workspace_statuses: Vec<WorkspaceWithStatus> = Vec::new();
    for workspace in workspaces {
        let details = get_workspace_status_details(&workspace, &werx)?;
        workspace_statuses.push(WorkspaceWithStatus { workspace, details });
    }

    // Filter to only workspaces matching the requested conditions
    let matching: Vec<&WorkspaceWithStatus> = workspace_statuses
        .iter()
        .filter(|ws| {
            (check_uncommitted && ws.details.uncommitted_changes)
                || (check_unpushed && ws.details.unpushed_branch)
                || (check_merged && ws.details.merged_branch)
        })
        .collect();

    match format.as_str() {
        "json" => {
            print_check_json(&matching, check_uncommitted, check_unpushed, check_merged)?;
        }
        _ => {
            print_check_text(&matching, check_uncommitted, check_unpushed, check_merged)?;
        }
    }

    Ok(())
}

fn print_check_text(
    matching: &[&WorkspaceWithStatus],
    check_uncommitted: bool,
    check_unpushed: bool,
    check_merged: bool,
) -> Result<()> {
    if matching.is_empty() {
        let mut checks = Vec::new();
        if check_uncommitted {
            checks.push("uncommitted changes");
        }
        if check_unpushed {
            checks.push("unpushed branches");
        }
        if check_merged {
            checks.push("merged branches");
        }
        println!("No workspaces found with {}.", checks.join(", "));
        return Ok(());
    }

    println!();

    // Group by condition type
    if check_uncommitted {
        let uncommitted: Vec<_> = matching
            .iter()
            .filter(|w| w.details.uncommitted_changes)
            .collect();
        if !uncommitted.is_empty() {
            println!("Uncommitted Changes ({}):", uncommitted.len());
            for ws in &uncommitted {
                let change_summary = if let Some(ref details) = ws.details.status_details {
                    let mut parts = Vec::new();
                    if !details.modified_files.is_empty() {
                        parts.push(format!("M:{}", details.modified_files.len()));
                    }
                    if !details.untracked_files.is_empty() {
                        parts.push(format!("?:{}", details.untracked_files.len()));
                    }
                    parts.join(" ")
                } else {
                    String::new()
                };
                println!(
                    "  {}/{}  {}",
                    ws.workspace.repository, ws.workspace.name, change_summary
                );
            }
            println!();
        }
    }

    if check_unpushed {
        let unpushed: Vec<_> = matching
            .iter()
            .filter(|w| w.details.unpushed_branch)
            .collect();
        if !unpushed.is_empty() {
            println!("Unpushed Branches ({}):", unpushed.len());
            for ws in &unpushed {
                let branch = ws.details.branch_name.as_deref().unwrap_or("(unknown)");
                println!(
                    "  {}/{}  Branch '{}' not on remote",
                    ws.workspace.repository, ws.workspace.name, branch
                );
            }
            println!();
        }
    }

    if check_merged {
        let merged: Vec<_> = matching
            .iter()
            .filter(|w| w.details.merged_branch)
            .collect();
        if !merged.is_empty() {
            println!("Merged Branches ({}):", merged.len());
            for ws in &merged {
                let branch = ws.details.branch_name.as_deref().unwrap_or("(unknown)");
                let default = ws.details.default_branch.as_deref().unwrap_or("main");
                println!(
                    "  {}/{}  Branch '{}' merged to '{}'",
                    ws.workspace.repository, ws.workspace.name, branch, default
                );
            }
            println!();
        }
    }

    Ok(())
}

fn print_check_json(
    matching: &[&WorkspaceWithStatus],
    check_uncommitted: bool,
    check_unpushed: bool,
    check_merged: bool,
) -> Result<()> {
    #[derive(serde::Serialize)]
    struct JsonOutput {
        checks_performed: ChecksPerformed,
        workspaces: Vec<WorkspaceCheckJson>,
        count: usize,
    }

    #[derive(serde::Serialize)]
    struct ChecksPerformed {
        uncommitted: bool,
        unpushed: bool,
        merged: bool,
    }

    #[derive(serde::Serialize)]
    struct WorkspaceCheckJson {
        name: String,
        path: String,
        repository: String,
        branch: Option<String>,
        uncommitted_changes: bool,
        unpushed_branch: bool,
        merged_branch: bool,
    }

    let workspaces: Vec<WorkspaceCheckJson> = matching
        .iter()
        .map(|ws| WorkspaceCheckJson {
            name: ws.workspace.name.clone(),
            path: ws.workspace.path.display().to_string(),
            repository: ws.workspace.repository.clone(),
            branch: ws.details.branch_name.clone(),
            uncommitted_changes: ws.details.uncommitted_changes,
            unpushed_branch: ws.details.unpushed_branch,
            merged_branch: ws.details.merged_branch,
        })
        .collect();

    let output = JsonOutput {
        checks_performed: ChecksPerformed {
            uncommitted: check_uncommitted,
            unpushed: check_unpushed,
            merged: check_merged,
        },
        workspaces,
        count: matching.len(),
    };

    let json = serde_json::to_string_pretty(&output)?;
    println!("{}", json);

    Ok(())
}

// =============================================================================
// Agent Commands
// =============================================================================

fn cmd_agent_spawn(
    repo_spec: Option<String>,
    branch: Option<String>,
    agent: Option<String>,
    prompt: Option<String>,
    edit_prompt: bool,
) -> Result<()> {
    // Find the Werx
    let werx = find_werx()?;

    // Resolve repository
    let repo_info = if let Some(spec) = repo_spec {
        find_repository(&werx, &spec)?
    } else {
        // Try to detect current workspace first
        let current_dir = std::env::current_dir()?;
        if let Some(repo) = detect_current_workspace(&current_dir, &werx)? {
            println!();
            println!(
                "Using repository from current workspace: {}",
                repo.clone_url
            );
            repo
        } else {
            // Interactive fuzzy selector
            match fuzzy_select_repository(&werx)? {
                Some(repo) => repo,
                None => {
                    println!("Cancelled.");
                    return Ok(());
                }
            }
        }
    };

    // Parse agent type if specified
    let agent_type = if let Some(agent_str) = agent {
        Some(agent_str.parse::<AgentType>()?)
    } else {
        None
    };

    // Handle branch - if not specified, prompt for a new branch name
    let (final_branch, base_branch) = if let Some(b) = branch {
        (Some(b), None)
    } else {
        // Prompt for a new branch name with option to change base branch
        let (new_branch, base) = prompt_branch_name(&werx, &repo_info)?;
        (Some(new_branch), Some(base))
    };

    // Handle edit-prompt
    let final_prompt = if edit_prompt {
        Some(get_prompt_from_editor()?)
    } else {
        prompt
    };

    // Build spawn options
    let options = SpawnOptions {
        agent_type,
        branch: final_branch,
        base_branch,
        prompt: final_prompt,
    };

    println!();
    println!("Spawning agent for {}...", repo_info.dir_name);

    // Spawn the agent
    let result = spawn_agent(&werx, &repo_info, options)?;

    println!();
    println!("Agent spawned successfully!");
    println!();
    println!("  Name:      {}", result.agent.name);
    println!("  Agent:     {}", result.agent.agent_type);
    println!("  Repo:      {}", result.agent.repository);
    if let Some(branch) = &result.agent.branch {
        println!("  Branch:    {}", branch);
    }
    println!("  Worktree:  {}", result.agent.worktree_path.display());
    if let Some(created_reason) = &result.created_branch {
        println!();
        println!("  Note: {}", created_reason);
    }
    println!();
    println!("To interact with the agent:");
    println!("  {}", result.attach_command);
    println!();

    Ok(())
}

fn cmd_agent_list(format: String) -> Result<()> {
    let werx = find_werx()?;
    let agents = list_agents(&werx)?;

    if agents.is_empty() {
        println!("No agents are currently running.");
        println!();
        println!("Run 'werx agent spawn' to start an agent.");
        return Ok(());
    }

    match format.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&agents)?;
            println!("{}", json);
        }
        _ => {
            println!();
            println!("Running Agents:");
            println!();

            for agent in &agents {
                let status_indicator = match agent.status {
                    werx::agent::AgentStatus::Running => "*",
                    werx::agent::AgentStatus::Exited => " ",
                    werx::agent::AgentStatus::Failed => "!",
                    werx::agent::AgentStatus::Unknown => "?",
                };

                println!(
                    "  {} {}  ({}) - {}",
                    status_indicator, agent.name, agent.agent_type, agent.repository
                );
                if let Some(branch) = &agent.branch {
                    println!("      Branch: {}", branch);
                }
            }

            println!();
            println!("Total: {} agents", agents.len());
            println!();
            println!("Commands:");
            println!("  werx agent attach [name]  - Attach to agent session");
            println!("  werx agent kill [name]    - Kill an agent");
            println!();
        }
    }

    Ok(())
}

fn cmd_agent_status(agent_name: Option<String>, format: String) -> Result<()> {
    let werx = find_werx()?;

    let agents = if let Some(name) = agent_name {
        let agent = find_agent(&werx, &name)?;
        vec![agent]
    } else {
        list_agents(&werx)?
    };

    if agents.is_empty() {
        println!("No agents are currently running.");
        return Ok(());
    }

    match format.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&agents)?;
            println!("{}", json);
        }
        _ => {
            println!();
            for agent in &agents {
                println!("Agent: {}", agent.name);
                println!("  Type:      {}", agent.agent_type);
                println!("  Status:    {}", agent.status);
                println!("  Repo:      {}", agent.repository);
                if let Some(branch) = &agent.branch {
                    println!("  Branch:    {}", branch);
                }
                println!("  Worktree:  {}", agent.worktree_path.display());
                println!();
            }
        }
    }

    Ok(())
}

fn cmd_agent_attach(agent_name: Option<String>) -> Result<()> {
    let werx = find_werx()?;

    // If no agent specified and multiple exist, show interactive selector
    let agent_to_attach = if agent_name.is_none() {
        let agents = list_agents(&werx)?;
        if agents.is_empty() {
            return Err(anyhow::anyhow!(
                "No agents are currently running.\n\nRun 'werx agent spawn' to start an agent."
            ));
        }
        if agents.len() == 1 {
            Some(agents[0].name.clone())
        } else if std::io::IsTerminal::is_terminal(&std::io::stdin()) {
            // Interactive selection
            println!();
            println!("Multiple agents running. Select one:");
            println!();

            let items: Vec<String> = agents.iter().map(|a| a.display()).collect();

            let selection =
                dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
                    .items(&items)
                    .default(0)
                    .interact()?;

            Some(agents[selection].name.clone())
        } else {
            return Err(anyhow::anyhow!(
                "Multiple agents running. Please specify which agent to attach to.\n\n\
                 Run 'werx agent list' to see running agents."
            ));
        }
    } else {
        agent_name
    };

    // This will exec into tmux, replacing the current process
    attach_to_agent(agent_to_attach.as_deref())
}

fn cmd_agent_kill(agent_name: Option<String>, cleanup: bool) -> Result<()> {
    let werx = find_werx()?;

    // If no agent specified, show interactive selector or error
    let agent_to_kill = if let Some(name) = agent_name {
        name
    } else {
        let agents = list_agents(&werx)?;
        if agents.is_empty() {
            return Err(anyhow::anyhow!("No agents are currently running."));
        }
        if agents.len() == 1 {
            agents[0].name.clone()
        } else if std::io::IsTerminal::is_terminal(&std::io::stdin()) {
            // Interactive selection
            println!();
            println!("Multiple agents running. Select one to kill:");
            println!();

            let items: Vec<String> = agents.iter().map(|a| a.display()).collect();

            let selection =
                dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
                    .items(&items)
                    .default(0)
                    .interact()?;

            agents[selection].name.clone()
        } else {
            return Err(anyhow::anyhow!(
                "Multiple agents running. Please specify which agent to kill.\n\n\
                 Run 'werx agent list' to see running agents."
            ));
        }
    };

    // Get agent info before killing
    let agent = find_agent(&werx, &agent_to_kill)?;

    println!();
    println!("Killing agent '{}'...", agent.name);

    let session_closed = kill_agent(&werx, &agent.name, cleanup)?;

    println!();
    println!("Agent '{}' terminated.", agent.name);

    if cleanup {
        println!("Worktree removed: {}", agent.worktree_path.display());
    } else {
        println!("Worktree preserved: {}", agent.worktree_path.display());
        println!();
        println!("To remove the worktree later:");
        println!(
            "  werx workspace remove {}/{}",
            agent.repository, agent.name
        );
    }

    if session_closed {
        println!();
        println!("Note: This was the last agent, tmux session has been closed.");
    }

    println!();

    Ok(())
}

fn cmd_agent_providers() -> Result<()> {
    let providers = detect_providers();
    let default = get_default_provider(&providers, None);

    println!();
    println!("Available Agent Providers:");
    println!();

    for provider in &providers {
        let status = if provider.available { "*" } else { " " };
        let is_default = if let Ok(d) = &default {
            d.agent_type == provider.agent_type
        } else {
            false
        };
        let default_marker = if is_default { " (default)" } else { "" };

        println!(
            "  {} {}{}",
            status,
            provider.agent_type.display_name(),
            default_marker
        );
        println!("      Command: {}", provider.full_command());
        if let Some(path) = &provider.path {
            println!("      Path:    {}", path);
        }
        println!(
            "      Status:  {}",
            if provider.available {
                "Available"
            } else {
                "Not found"
            }
        );
        println!();
    }

    println!("Legend: * = Available");
    println!();

    if default.is_err() {
        println!("Warning: No agents are available.");
        println!();
        println!("Install one of the following:");
        println!("  - OpenCode: https://opencode.ai");
        println!("  - Claude Code: https://claude.ai/code");
        println!("  - GitHub Copilot CLI: gh extension install github/gh-copilot");
        println!();
    }

    Ok(())
}

/// Open $EDITOR to compose a prompt
fn get_prompt_from_editor() -> Result<String> {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());

    // Create a temp file
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join("werx-agent-prompt.txt");

    // Write a placeholder
    std::fs::write(
        &temp_file,
        "# Enter your prompt for the agent below.\n# Lines starting with # will be ignored.\n\n",
    )?;

    // Open editor
    let status = std::process::Command::new(&editor)
        .arg(&temp_file)
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to open editor '{}': {}", editor, e))?;

    if !status.success() {
        return Err(anyhow::anyhow!("Editor exited with non-zero status"));
    }

    // Read the file
    let content = std::fs::read_to_string(&temp_file)?;

    // Clean up
    let _ = std::fs::remove_file(&temp_file);

    // Filter out comment lines and trim
    let prompt: String = content
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string();

    if prompt.is_empty() {
        return Err(anyhow::anyhow!("Prompt is empty. Aborting spawn."));
    }

    Ok(prompt)
}
