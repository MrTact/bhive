//! B'hive CLI
//!
//! Command-line client for the B'hive API.

use anyhow::Result;
use clap::{Parser, Subcommand};

mod client;
mod commands;

use client::ApiClient;

#[derive(Parser)]
#[command(name = "bhive")]
#[command(about = "B'hive CLI - Massively parallel AI agent orchestration")]
#[command(version)]
struct Cli {
    /// API server URL
    #[arg(long, env = "BHIVE_API_URL", default_value = "http://localhost:3030")]
    api_url: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize bhive for this project
    Init {
        /// Database URL (defaults to local PostgreSQL)
        #[arg(long, env = "DATABASE_URL")]
        database_url: Option<String>,

        /// Force re-initialization
        #[arg(long)]
        force: bool,
    },

    /// Create a new task
    Task {
        #[command(subcommand)]
        action: TaskCommands,
    },

    /// Manage workers
    Workers {
        #[command(subcommand)]
        action: WorkerCommands,
    },

    /// Queen agent status
    Queen {
        #[command(subcommand)]
        action: QueenCommands,
    },
}

#[derive(Subcommand)]
enum TaskCommands {
    /// Create a new task
    Create {
        /// Task description
        description: String,

        /// Files to operate on (glob patterns)
        #[arg(long, short = 'f')]
        files: Vec<String>,

        /// Maximum number of workers
        #[arg(long, default_value = "10")]
        max_workers: u32,

        /// Generation provider (format: provider/model)
        #[arg(long, default_value = "openai/gpt-4o")]
        generate: String,

        /// Review provider (format: provider/model)
        #[arg(long)]
        review: Option<String>,
    },

    /// Get task status
    Status {
        /// Task ID
        task_id: String,
    },

    /// Watch task progress (streaming)
    Watch {
        /// Task ID
        task_id: String,
    },

    /// List all tasks
    List,
}

#[derive(Subcommand)]
enum WorkerCommands {
    /// List all workers
    List,

    /// Get worker status
    Status {
        /// Worker ID
        worker_id: String,
    },
}

#[derive(Subcommand)]
enum QueenCommands {
    /// Get queen status
    Status,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Init {
            database_url,
            force,
        } => {
            commands::init::run(database_url, force).await?;
        }
        Commands::Task { action } => {
            // Get current project
            let project = commands::project::get_current_project()?;
            let client = ApiClient::new(&cli.api_url).with_project_id(project.project_id.clone());

            // Update last seen
            let _ = commands::project::update_project_last_seen();

            match action {
            TaskCommands::Create {
                description,
                files,
                max_workers,
                generate,
                review,
            } => {
                commands::task::create(
                    &client,
                    description,
                    files,
                    max_workers,
                    generate,
                    review,
                )
                .await?;
            }
            TaskCommands::Status { task_id } => {
                commands::task::status(&client, &task_id).await?;
            }
            TaskCommands::Watch { task_id } => {
                commands::task::watch(&client, &task_id).await?;
            }
            TaskCommands::List => {
                commands::task::list(&client).await?;
            }
            }
        }
        Commands::Workers { action } => {
            // Get current project
            let project = commands::project::get_current_project()?;
            let client = ApiClient::new(&cli.api_url).with_project_id(project.project_id.clone());

            // Update last seen
            let _ = commands::project::update_project_last_seen();

            match action {
            WorkerCommands::List => {
                commands::worker::list(&client).await?;
            }
            WorkerCommands::Status { worker_id } => {
                commands::worker::status(&client, &worker_id).await?;
            }
            }
        }
        Commands::Queen { action } => {
            // Get current project
            let project = commands::project::get_current_project()?;
            let client = ApiClient::new(&cli.api_url).with_project_id(project.project_id.clone());

            // Update last seen
            let _ = commands::project::update_project_last_seen();

            match action {
            QueenCommands::Status => {
                commands::queen::status(&client).await?;
            }
            }
        }
    }

    Ok(())
}
