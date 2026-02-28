//! Initialize bhive for a project

use anyhow::{Context, Result};
use bhive_core::{
    coordination::Coordinator,
    project::{ProjectConfig, ProjectRegistry},
};
use std::env;

/// Run the init command
pub async fn run(database_url: Option<String>, force: bool) -> Result<()> {
    println!("🐝 Initializing B'hive for this project...\n");

    // Get current directory
    let current_dir = env::current_dir().context("Failed to get current directory")?;
    println!("📂 Project directory: {}", current_dir.display());

    // Load or create project registry
    let mut registry = ProjectRegistry::load().unwrap_or_else(|_| {
        println!("📋 Creating new project registry...");
        ProjectRegistry::default()
    });

    // Check if project is already registered
    if let Some(existing) = registry.get_by_path(&current_dir) {
        if !force {
            println!("✅ Project already initialized!");
            println!("   Project ID: {}", existing.project_id);
            println!("   Database: {}", existing.db_name);
            println!("   Last seen: {}", existing.last_seen);
            println!("\nUse --force to re-initialize.");
            return Ok(());
        }
        println!("⚠️  Re-initializing existing project (--force)");
    }

    // Create project config
    let config = ProjectConfig::new(&current_dir);
    println!("🆔 Project ID: {}", config.project_id);
    println!("💾 Database: {}", config.db_name);

    // Determine database URL
    let db_url = database_url.unwrap_or_else(|| {
        format!(
            "postgresql://bhive:bhive_dev@localhost:5432/postgres"
        )
    });

    // Connect to PostgreSQL
    println!("\n🔌 Connecting to PostgreSQL...");
    let pool = sqlx::PgPool::connect(&db_url)
        .await
        .context("Failed to connect to PostgreSQL. Is the Docker infrastructure running?")?;

    println!("✅ Connected to PostgreSQL");

    // Check if project database exists
    println!("\n🗄️  Checking project database...");
    let db_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = $1)"
    )
    .bind(&config.db_name)
    .fetch_one(&pool)
    .await?;

    if db_exists {
        println!("✅ Database '{}' already exists", config.db_name);
    } else {
        // Create database from template
        println!("📦 Creating database '{}'...", config.db_name);

        // Check if template exists
        let template_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = 'bhive_template')"
        )
        .fetch_one(&pool)
        .await?;

        if !template_exists {
            anyhow::bail!(
                "Template database 'bhive_template' not found. \
                Please run Docker infrastructure first: cd docker && docker-compose up -d"
            );
        }

        // Create database from template
        let create_query = format!(
            "CREATE DATABASE {} TEMPLATE bhive_template",
            config.db_name
        );
        sqlx::query(&create_query)
            .execute(&pool)
            .await
            .context("Failed to create project database")?;

        println!("✅ Database created successfully");
    }

    // Connect to project database
    let project_db_url = db_url.replace("/postgres", &format!("/{}", config.db_name));
    let coordinator = Coordinator::new(&project_db_url)
        .await
        .context("Failed to connect to project database")?;

    // Check if schema already exists (from template)
    println!("\n📝 Checking schema...");
    let schema_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM information_schema.tables WHERE table_name = 'operators')"
    )
    .fetch_one(coordinator.pool())
    .await?;

    if schema_exists {
        println!("✅ Schema already initialized (from template)");
    } else {
        println!("📝 Running migrations...");
        coordinator
            .migrate()
            .await
            .context("Failed to run migrations")?;
        println!("✅ Migrations complete");
    }

    // Register project
    println!("\n📝 Registering project...");
    registry.register(config.clone())?;
    registry.save()?;

    println!("✅ Project registered");

    // Print summary
    println!("\n🎉 B'hive initialization complete!");
    println!("\nProject Details:");
    println!("  ID:       {}", config.project_id);
    println!("  Path:     {}", config.path.display());
    println!("  Database: {}", config.db_name);
    println!("  Qdrant:   {}", config.qdrant_collection);
    println!("  Redis:    {}", config.redis_prefix);

    println!("\nConnection String:");
    println!("  {}", project_db_url);

    println!("\nNext Steps:");
    println!("  1. Create a task: bhive task create \"Your task description\"");
    println!("  2. Check status:  bhive task list");
    println!("  3. View workers:  bhive workers list");

    Ok(())
}
