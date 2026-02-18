//! Ant Army CLI
//!
//! Command-line client for the Ant Army API.

use anyhow::Result;
use clap::{Parser, Subcommand};

mod client;
mod commands;

use client::ApiClient;

#[derive(Parser)]
#[command(name = "ant-army")]
#[command(about = "Ant Army CLI - Massively parallel AI agent orchestration")]
#[command(version)]
struct Cli {
    /// API server URL
    #[arg(long, env = "ANT_ARMY_API_URL", default_value = "http://localhost:3030")]
    api_url: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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
    let client = ApiClient::new(&cli.api_url);

    match cli.command {
        Commands::Task { action } => match action {
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
        },
        Commands::Workers { action } => match action {
            WorkerCommands::List => {
                commands::worker::list(&client).await?;
            }
            WorkerCommands::Status { worker_id } => {
                commands::worker::status(&client, &worker_id).await?;
            }
        },
        Commands::Queen { action } => match action {
            QueenCommands::Status => {
                commands::queen::status(&client).await?;
            }
        },
    }

    Ok(())
}
