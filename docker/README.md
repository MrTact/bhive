# B'hive Docker Infrastructure

This directory contains the shared Docker infrastructure for all bhive projects.

## Overview

The bhive coordination system uses a **single shared Docker Compose stack** that serves all projects. Each project gets its own isolated database within a single PostgreSQL instance.

### Architecture

```
┌─────────────────────────────────────────┐
│       Docker Compose Stack              │
├─────────────────────────────────────────┤
│  API Server (Port 3030)                 │
│  └─ REST/WebSocket endpoints            │
│                                          │
│  PostgreSQL Instance (Port 5432)        │
│  ├─ bhive_project_a (DB)            │
│  ├─ bhive_project_b (DB)            │
│  └─ bhive_project_c (DB)            │
│                                          │
│  Future Services:                        │
│  ├─ Qdrant (Vector DB)                 │
│  └─ Redis (Caching)                    │
└─────────────────────────────────────────┘
```

### Data Storage

All coordination data is stored centrally at:
```
~/.config/bhive/data/
├── postgres/          # PostgreSQL data
├── qdrant/           # Vector embeddings (future)
└── redis/            # Cache data (future)
```

## Quick Start

### 1. Start the Infrastructure

```bash
cd docker
docker-compose up -d
```

This will:
- Start PostgreSQL on port 5432
- Start API Server on port 3030
- Create the `bhive_template` database with the coordination schema
- Set up helper functions for creating project databases

### 2. Verify Services

```bash
docker-compose ps
```

Expected output:
```
NAME                  STATUS    PORTS
bhive-api          Up        0.0.0.0:3030->3030/tcp
bhive-postgres     Up        0.0.0.0:5432->5432/tcp
```

### 3. View Logs

```bash
docker-compose logs -f postgres
```

## Project Database Creation

Project databases are created automatically when you run `bhive init` in a project directory. The initialization process:

1. Generates a stable project ID (e.g., `my_app_a1b2`)
2. Connects to PostgreSQL
3. Creates database `bhive_my_app_a1b2` from the template
4. Registers the project in `~/.config/bhive/projects.toml`

### Manual Database Creation

To manually create a project database:

```bash
# Connect to PostgreSQL
psql -h localhost -U bhive -d postgres

# Create project database
SELECT create_project_database('my_project_id');
```

The function returns `true` if the database was created, `false` if it already exists.

## Configuration

### Environment Variables

Create a `.env` file from the example:

```bash
cp .env.example .env
```

Available variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `POSTGRES_USER` | `bhive` | PostgreSQL username |
| `POSTGRES_PASSWORD` | `bhive_dev` | PostgreSQL password |
| `POSTGRES_PORT` | `5432` | PostgreSQL port |
| `API_PORT` | `3030` | API server port |
| `RUST_LOG` | `bhive_api=debug,tower_http=debug` | Logging configuration |
| `BHIVE_DATA_DIR` | `~/.config/bhive/data` | Data storage location |

### Connection String

Projects connect using:
```
postgresql://bhive:bhive_dev@localhost:5432/bhive_{project_id}
```

Example for project `my_app_a1b2`:
```
postgresql://bhive:bhive_dev@localhost:5432/bhive_my_app_a1b2
```

## Management Commands

### Start Services
```bash
docker-compose up -d
```

### Stop Services
```bash
docker-compose down
```

### Restart Services
```bash
docker-compose restart
```

### View Status
```bash
docker-compose ps
```

### View Logs
```bash
# All services
docker-compose logs -f

# Specific service
docker-compose logs -f postgres
```

### Connect to PostgreSQL
```bash
# Using psql from host (requires psql installed)
psql -h localhost -U bhive -d postgres

# Using psql from container
docker exec -it bhive-postgres psql -U bhive -d postgres
```

### List Project Databases
```bash
psql -h localhost -U bhive -d postgres -c "\l bhive_*"
```

### Backup Project Database
```bash
docker exec bhive-postgres pg_dump -U bhive bhive_my_project > backup.sql
```

### Restore Project Database
```bash
cat backup.sql | docker exec -i bhive-postgres psql -U bhive bhive_my_project
```

## Database Schema

The template database includes:

### Tables
- **bees**: Worker bee pool (idle/active/failed states)
- **tasks**: Task queue with dependency tracking
- **task_dependencies**: Task DAG structure
- **logs**: Event and error logging

### Functions
- `acquire_bee(bee_type)`: Get idle bee or create new one
- `release_bee(bee_id, success)`: Return bee to pool
- `claim_task(task_id, bee_id)`: Atomically claim a task
- `get_ready_tasks()`: Find tasks ready for execution
- `log_event(...)`: Log coordination events

### Types
- `bee_type`: operator, analyst, builder, tester
- `bee_status`: idle, active, failed
- `task_status`: pending, claimed, active, completed, failed, blocked
- `log_level`: debug, info, warn, error

## Multi-Project Scenarios

### Scenario 1: Multiple Projects on Same Machine

```
/home/user/project-a/  → bhive_project_a_a1b2
/home/user/project-b/  → bhive_project_b_c3d4
/home/user/project-c/  → bhive_project_c_e5f6
```

All projects connect to the same PostgreSQL instance but have isolated databases.

### Scenario 2: Project Moved to New Location

When you move a project directory, the project ID remains stable (based on original path hash). The registry tracks the new location:

```
Old: /home/user/old-path/my-app  → my_app_a1b2
New: /home/user/new-path/my-app  → my_app_a1b2 (same ID)
```

### Scenario 3: Multiple Checkouts of Same Project

Different working directories of the same codebase get unique project IDs:

```
/home/user/main/my-app     → my_app_a1b2
/home/user/feature/my-app  → my_app_c3d4
```

This allows concurrent work without database conflicts.

## Troubleshooting

### Port Already in Use

If port 5432 is already in use:

1. Check for existing PostgreSQL:
   ```bash
   lsof -i :5432
   ```

2. Either stop the existing service or change the port in `.env`:
   ```env
   POSTGRES_PORT=5433
   ```

### Cannot Connect to Database

1. Check if services are running:
   ```bash
   docker-compose ps
   ```

2. Check logs for errors:
   ```bash
   docker-compose logs postgres
   ```

3. Test connection:
   ```bash
   psql -h localhost -U bhive -d postgres -c "SELECT 1"
   ```

### Database Not Created

If a project database wasn't created automatically:

```bash
# Check if template exists
psql -h localhost -U bhive -d postgres -c "\l bhive_template"

# Manually create the database
psql -h localhost -U bhive -d postgres -c "SELECT create_project_database('your_project_id');"
```

### Clean Slate

To completely reset the infrastructure:

```bash
# Stop and remove containers, networks, volumes
docker-compose down -v

# Remove data directory (CAUTION: deletes all project data!)
rm -rf ~/.config/bhive/data/postgres

# Start fresh
docker-compose up -d
```

## Future Services

### Qdrant (Vector Database)

For LEGOMem context storage, uncomment the Qdrant service in `docker-compose.yml`:

```yaml
qdrant:
  image: qdrant/qdrant:latest
  container_name: bhive-qdrant
  restart: unless-stopped
  ports:
    - "${QDRANT_PORT:-6333}:6333"
  volumes:
    - ${BHIVE_DATA_DIR:-~/.config/bhive/data/qdrant}:/qdrant/storage
  networks:
    - bhive
```

### Redis (Caching)

For performance optimization, uncomment the Redis service:

```yaml
redis:
  image: redis:7-alpine
  container_name: bhive-redis
  restart: unless-stopped
  ports:
    - "${REDIS_PORT:-6379}:6379"
  volumes:
    - ${BHIVE_DATA_DIR:-~/.config/bhive/data/redis}:/data
  networks:
    - bhive
```

## Development

### Modifying the Schema

To modify the database schema:

1. Update `init-db.sh` with your changes
2. Stop the stack: `docker-compose down -v`
3. Remove the data directory (or just the PostgreSQL data)
4. Restart: `docker-compose up -d`

**Note**: Existing project databases will need to be recreated or migrated.

### Adding Migrations

For production-ready migration support:

1. Create migration files in `docker/migrations/`
2. Mount the migrations directory in `docker-compose.yml`
3. Use a migration tool like `sqlx migrate` or `refinery`

Example migration structure:
```
docker/migrations/
├── 001_initial_schema.sql
├── 002_add_indexes.sql
└── 003_add_metrics.sql
```

## Architecture Decisions

### Why Single Stack?

- **Simplicity**: One `docker-compose up` for all projects
- **Resource Efficiency**: Shared PostgreSQL instance instead of N instances
- **Port Management**: No port allocation complexity
- **Easy Monitoring**: Single set of logs and metrics

### Why Per-Project Databases?

- **Isolation**: Projects don't interfere with each other
- **Cleanup**: Easy to remove project-specific data
- **Concurrency**: Multiple projects can run simultaneously
- **Flexibility**: Different schema versions possible (with migrations)

### Why Centralized Storage?

- **PostgreSQL Architecture**: Stores all databases in one data directory
- **Reliability**: Moving project directories doesn't break database access
- **Backup**: Single location to back up all coordination data
- **Permissions**: Consistent ownership and access control

## See Also

- [Coordination Layer Architecture](../docs/bhive/COORDINATION_LAYER.md)
- [Project Registry](../docs/bhive/PROJECT_REGISTRY.md)
- [Setup Guide](../docs/bhive/SETUP.md)
