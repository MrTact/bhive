# B'hive Setup Tool - Mini PRD

**Status:** Planning
**Last Updated:** February 16, 2026

---

## Problem

B'hive creates sibling workspace directories next to the user's repository for parallel operator execution. Most developers organize their repos under a single parent directory (e.g., `~/git-repos/`), which creates issues:

1. **Pollution** - Workspace directories (`brave-lion/`, `cosmic-bee/`) mix with unrelated projects
2. **Name collisions** - Operator names from different projects could conflict
3. **Confusion** - Users don't expect bhive to create directories outside their repo

## Solution

A **setup tool** that runs automatically when a user first activates B'hive, ensuring their environment is properly configured before spawning any operators.

---

## Trigger Mechanism

**Recommended:** Integrate with the `/switch` agent command.

When the user switches to the `queen` agent (via `/switch queen` or TUI agent picker):

1. Check if B'hive setup has been completed for this project
2. If not, run the setup tool sequence before activating queen mode
3. Store completion state in project config (`.opencode/bhive.json`)

```
User: /switch queen

[B'hive Setup]
This is your first time using B'hive in this project.
Running setup checks...

✓ VCS detected: jujutsu
✗ Workspace directory check failed

Your repository is at: ~/git-repos/my-project/
Parent directory contains 12 other items.

B'hive needs a clean parent directory to create workspace siblings.
Recommended structure:
  ~/git-repos/my-project/
  └── repo/          <- move your repo here
  └── workspaces/    <- operator workspaces go here

Would you like help reorganizing? (y/n)
```

---

## Setup Tool Architecture

### Sequence Runner

The setup tool runs a series of checks in order. Each check can:

- **Pass** - Continue to next check
- **Fail with fix** - Prompt user, offer remediation
- **Fail hard** - Block activation until resolved

```typescript
interface SetupCheck {
  name: string
  description: string
  run(): Promise<SetupResult>
}

type SetupResult =
  | { status: "pass" }
  | { status: "fail"; message: string; fix?: () => Promise<boolean> }
  | { status: "skip"; reason: string }
```

### Initial Check: Clean Parent Directory

**Purpose:** Ensure the repository's parent directory is suitable for workspace siblings.

**Logic:**

```
1. Get repo root: Instance.worktree
2. Get parent: path.dirname(Instance.worktree)
3. List parent contents (excluding repo itself)
4. If empty or only contains "workspaces/": PASS
5. If contains other items: FAIL with remediation prompt
```

**Remediation options:**

1. **Move repo deeper** (recommended)
   - Suggest subdirectory name based on repo's default branch/bookmark (e.g., `main`, `dev`, `trunk`)
   - Perform atomic relocation (see Relocation Process below)
   - Update user's working directory
2. **Use alternate workspace location** (advanced)
   - Allow user to specify custom `workspacesRoot` in config
   - Not recommended (breaks the "sibling" model)

3. **Proceed anyway** (dangerous)
   - User accepts pollution risk
   - Store acknowledgment in config

### Relocation Process

When moving a repo deeper, use this atomic process to avoid data loss:

```
Initial state:
  ~/git-repos/opencode/     <- repo root (default branch: "dev")

Step 1: Create sibling temp directory
  ~/git-repos/opencode/
  ~/git-repos/temp/

Step 2: Move repo into temp, rename to branch name
  ~/git-repos/temp/dev/     <- former repo root

Step 3: Rename temp to original repo name
  ~/git-repos/opencode/dev/ <- repo now lives here

Step 4: If cwd was repo root, cd to new location
  cd ~/git-repos/opencode/  <- user is now in wrapper directory
```

**Subdirectory naming:**

- Query VCS for default branch/bookmark name
- Git: `git symbolic-ref refs/remotes/origin/HEAD` → `refs/remotes/origin/dev` → `dev`
- Jujutsu: Check for common bookmark names (`main`, `master`, `trunk`, `dev`)
- Fallback: `repo` if detection fails
- Allow user to override suggested name

**Example scenarios:**
| Repo | Default Branch | Suggested Structure |
|------|----------------|---------------------|
| opencode | `dev` | `opencode/dev/` |
| my-app | `main` | `my-app/main/` |
| legacy-system | `trunk` | `legacy-system/trunk/` |

---

## Future Setup Checks (Planned)

The setup tool is designed to be extensible. Anticipated checks:

| Check                  | Purpose                                  | Phase |
| ---------------------- | ---------------------------------------- | ----- |
| Clean parent directory | Workspace isolation                      | 1     |
| VCS availability       | Ensure jj/git installed                  | 1     |
| VCS initialization     | Ensure repo is initialized               | 1     |
| Disk space             | Warn if < 5GB available                  | 2     |
| Config validation      | Validate bhive settings                  | 2     |
| Model access           | Verify API keys for configured providers | 2     |
| Workspace cleanup      | Offer to clean stale workspaces          | 3     |

---

## Configuration Storage

### Project-level state

`.opencode/bhive.json`:

```json
{
  "setupCompleted": true,
  "setupVersion": 1,
  "completedAt": "2026-02-16T10:30:00Z",
  "checks": {
    "cleanParentDirectory": {
      "status": "pass",
      "acknowledgedRisk": false
    }
  },
  "workspacesRoot": "../workspaces"
}
```

### Re-running Setup

Users can re-run setup via:

- `/bhive setup` command
- Automatic re-run when `setupVersion` < current version
- Manual deletion of `.opencode/bhive.json`

---

## User Experience Flow

### Happy Path (Clean Environment)

```
User: /switch queen

[B'hive Setup]
Running first-time setup checks...

✓ VCS: jujutsu detected
✓ Parent directory: clean (ready for workspaces)
✓ Configuration: valid

Setup complete! Queen agent activated.
You can now spawn operators for parallel development.

Queen: How can I help you today?
```

### Needs Reorganization

```
User: /switch queen

[B'hive Setup]
Running first-time setup checks...

✓ VCS: jujutsu detected
✗ Parent directory contains other projects

Your repo: ~/git-repos/opencode/
Default branch: dev
Parent contents:
  - other-project/
  - another-repo/
  - notes.txt

B'hive creates workspace directories as siblings to your repo.
This would pollute ~/git-repos/ with operator workspaces.

Recommended fix: Move repo to ~/git-repos/opencode/dev/
  ~/git-repos/opencode/
  └── dev/         <- your code (named after default branch)
  └── workspaces/  <- ant workspaces go here

Subdirectory name [dev]: _

[R]eorganize  [S]kip (use anyway)  [C]ancel
```

### User Chooses Reorganize

```
User: r

Reorganizing...
  Creating ~/git-repos/temp/
  Moving repo to ~/git-repos/temp/dev/
  Renaming ~/git-repos/temp/ to ~/git-repos/opencode/

Done! Your repo is now at: ~/git-repos/opencode/dev/

Changing working directory...
Please restart opencode from the new location:
  cd ~/git-repos/opencode/dev && opencode

[Setup will continue on next launch]
```

---

## Implementation Notes

### File Structure

```
packages/opencode/src/
├── bhive/
│   ├── setup/
│   │   ├── index.ts          # Setup runner
│   │   ├── checks/
│   │   │   ├── clean-parent.ts
│   │   │   ├── vcs-available.ts
│   │   │   └── index.ts
│   │   └── state.ts          # Read/write setup state
│   └── index.ts
```

### Integration Points

1. **Agent switching** (`agent/agent.ts`)
   - Hook into agent activation
   - Run setup if switching to queen and not completed

2. **Worktree creation** (`worktree/index.ts`)
   - Update `storageRoot` to use sibling `workspaces/` directory
   - `path.join(path.dirname(Instance.worktree), "workspaces")`

3. **Configuration** (`config/config.ts`)
   - Add `bhive.workspacesRoot` option for advanced users

---

## Open Questions

1. **Should setup block queen activation or just warn?**
   - Recommendation: Block with option to acknowledge risk

2. **How to handle existing polluted environments?**
   - Show existing workspaces, offer cleanup

3. **Should we support multiple repos sharing a workspaces directory?**
   - Initial answer: No, keep it simple (one project per workspace root)
   - Could revisit if demand exists

4. **What about CI/CD environments?**
   - Setup should auto-pass in non-interactive mode
   - Or provide `--no-setup` flag

---

## Success Metrics

- 95%+ of users complete setup on first attempt
- < 5% of users choose "skip" (accept pollution)
- Zero support issues related to workspace location confusion
