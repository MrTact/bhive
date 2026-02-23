# Database Migrations

This directory contains SQL migrations managed by sqlx.

## Structure

Migrations are named: `{timestamp}_{description}.sql`

- `20240101000000_initial_schema.sql` - Initial coordination layer schema

## Running Migrations

### Prerequisites

Install sqlx-cli:
```bash
cargo install sqlx-cli --no-default-features --features postgres
```

### Development

1. **Create a new migration:**
   ```bash
   sqlx migrate add <description>
   ```

2. **Run pending migrations:**
   ```bash
   sqlx migrate run --database-url postgresql://ant_army:ant_army_dev@localhost:5432/ant_army_<project_id>
   ```

3. **Revert last migration:**
   ```bash
   sqlx migrate revert --database-url postgresql://ant_army:ant_army_dev@localhost:5432/ant_army_<project_id>
   ```

4. **Check migration status:**
   ```bash
   sqlx migrate info --database-url postgresql://ant_army:ant_army_dev@localhost:5432/ant_army_<project_id>
   ```

### Offline Mode (Compile-Time Verification)

sqlx can verify queries at compile time without a database connection:

1. **Prepare offline data:**
   ```bash
   cargo sqlx prepare --database-url postgresql://ant_army:ant_army_dev@localhost:5432/ant_army_test
   ```

2. **Build without database:**
   ```bash
   SQLX_OFFLINE=true cargo build
   ```

## Integration

Migrations are automatically run by the Coordinator when initializing a project database:

```rust
use sqlx::postgres::PgPool;
use sqlx::migrate::Migrator;

let migrator = Migrator::new(std::path::Path::new("./migrations")).await?;
migrator.run(&pool).await?;
```

## Project Database Setup

When a project is first initialized:

1. The project registry generates a stable project ID
2. The coordinator connects to the main PostgreSQL instance
3. A new database is created from the template (via Docker init script)
   OR migrations are run against a fresh database
4. The project is ready for coordination

## Migration Best Practices

1. **Never modify existing migrations** - Always create a new migration to change schema
2. **Test migrations thoroughly** - Test both up and down migrations
3. **Keep migrations small** - One logical change per migration
4. **Document breaking changes** - Add comments for significant schema changes
5. **Include rollback** - Ensure migrations can be reverted if needed

## Example: Adding a New Column

```sql
-- 20240102120000_add_ant_metrics.sql

-- Add metrics tracking to ants table
ALTER TABLE ants ADD COLUMN total_errors INTEGER NOT NULL DEFAULT 0;
ALTER TABLE ants ADD COLUMN average_task_duration_ms INTEGER;

-- Update index to include new metrics
CREATE INDEX idx_ants_metrics ON ants(total_errors, average_task_duration_ms);
```

## See Also

- [sqlx documentation](https://github.com/launchbadge/sqlx)
- [Migration guide](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md)
