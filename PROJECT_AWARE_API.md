# Project-Aware API Implementation

## Overview
The API server now requires a project ID to know which project database to connect to. This maintains clean separation between projects while allowing the API to serve multiple projects.

## Architecture

### Flow:
1. User runs `ant-army init` in project directory
2. Creates project database: `ant_army_<project_id>`
3. Registers in `~/.config/ant-army/projects.toml`
4. User runs `ant-army task create "foo"`
5. CLI looks up project from current directory
6. CLI sends `X-Project-ID` header with API request
7. API server connects to correct project database
8. Task is created in project-specific database

## Changes Made

### CLI (`ant-army-cli`)

**New module: `commands/project.rs`**
- `get_current_project()` - Detects and validates project initialization
- `update_project_last_seen()` - Updates project registry timestamps
- Returns helpful errors if project not initialized

**Updated: `client.rs`**
- Added `project_id` field to `ApiClient`
- `.with_project_id()` method to set project ID
- Automatically adds `X-Project-ID` header to all requests
- Improved error messages showing API response body

**Updated: `main.rs`**
- Check project initialization before non-init commands
- Add project ID to API client
- Update last-seen timestamp on each use

### API Server (`ant-army-api`)

**New module: `extractors.rs`**
- `ProjectId` extractor validates `X-Project-ID` header
- Returns `400 Bad Request` with helpful message if missing
- Explains how to initialize project

**Updated: `state.rs`**
- Connection pool cache per project
- `get_coordinator(project_id)` - Gets or creates coordinator for project
- Constructs project-specific database URLs: `ant_army_{project_id}`
- Maintains base connection for metadata operations

**Updated: `handlers.rs`**
- `create_task` now requires `ProjectId` extractor
- Gets project-specific coordinator from state
- Creates tasks in correct project database

## Error Messages

### CLI Errors

**Project not initialized:**
```
Error: Project not initialized in this directory.

Current directory: /path/to/project

To initialize ant-army for this project, run:
  ant-army init
```

**Registry not found:**
```
Error: Failed to load project registry. Have you run 'ant-army init' yet?
```

### API Errors

**Missing header (400):**
```
Missing X-Project-ID header.

This endpoint requires a project ID. Make sure you've run 'ant-army init'
in your project directory.

The CLI should automatically include this header. If you're calling the API
directly, add the header:
  X-Project-ID: your_project_id
```

**Database not found (500):**
```
Failed to connect to project database 'ant_army_myproject_a1b2'.
Has this project been initialized?
```

## Testing

### 1. Initialize a project
```bash
cd /path/to/your/project
ant-army init
```

### 2. Create a task
```bash
ant-army task create "Write hello world function"
```

### 3. Check API logs
```bash
docker-compose logs -f api
```

You should see:
```
Creating task in project myproject_a1b2: Write hello world function
Task created: <uuid>
```

### 4. Test error handling

**Without initialization:**
```bash
cd /tmp/uninitialized-project
ant-army task create "test"
```

Should get clear error about needing to run `ant-army init`.

**Direct API call without header:**
```bash
curl -X POST http://localhost:3030/api/v1/tasks \
  -H "Content-Type: application/json" \
  -d '{"description": "test", "files": [], "providers": {"generate": "openai/gpt-4o"}}'
```

Should get `400 Bad Request` with helpful message about missing header.

**Direct API call with header:**
```bash
curl -X POST http://localhost:3030/api/v1/tasks \
  -H "Content-Type: application/json" \
  -H "X-Project-ID: myproject_a1b2" \
  -d '{"description": "test", "files": [], "providers": {"generate": "openai/gpt-4o"}}'
```

Should successfully create task.

## Benefits

✅ Clean separation between projects
✅ Multiple projects can coexist
✅ API serves all projects with one instance
✅ Connection pooling per project
✅ Clear error messages guide users
✅ CLI handles project detection automatically
✅ Manual API calls are still possible with header
