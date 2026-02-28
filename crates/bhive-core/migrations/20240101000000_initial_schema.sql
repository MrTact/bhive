-- Initial schema for B'hive coordination layer
--
-- This migration creates the core tables and functions for:
-- - Worker pool management
-- - Task queue with dependency tracking
-- - Event logging
-- - Atomic coordination operations

-- Create extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Enum types
CREATE TYPE operator_type AS ENUM ('operator', 'analyst', 'builder', 'tester');
CREATE TYPE operator_status AS ENUM ('idle', 'active', 'failed');
CREATE TYPE task_status AS ENUM ('pending', 'claimed', 'active', 'completed', 'failed', 'blocked');
CREATE TYPE log_level AS ENUM ('debug', 'info', 'warn', 'error');

-- Operators table: Worker pool
CREATE TABLE operators (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    operator_type operator_type NOT NULL,
    status operator_status NOT NULL DEFAULT 'idle',
    workspace_path TEXT,
    current_task_id UUID,
    current_session_id TEXT,
    tasks_completed INTEGER NOT NULL DEFAULT 0,
    last_active_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Tasks table: Task queue
CREATE TABLE tasks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    description TEXT NOT NULL,
    status task_status NOT NULL DEFAULT 'pending',
    operator_id UUID REFERENCES operators(id),
    parent_id UUID REFERENCES tasks(id),
    session_id TEXT,
    result JSONB,
    error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    claimed_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ
);

-- Task dependencies: DAG structure
CREATE TABLE task_dependencies (
    task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    depends_on UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    PRIMARY KEY (task_id, depends_on)
);

-- Logs table: Event and error logging
CREATE TABLE logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    level log_level NOT NULL,
    operator_id UUID REFERENCES operators(id),
    task_id UUID REFERENCES tasks(id),
    message TEXT NOT NULL,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_operators_status ON operators(status);
CREATE INDEX idx_operators_type_status ON operators(operator_type, status);
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_operator_id ON tasks(operator_id);
CREATE INDEX idx_tasks_parent_id ON tasks(parent_id);
CREATE INDEX idx_logs_created_at ON logs(created_at DESC);
CREATE INDEX idx_logs_operator_id ON logs(operator_id);
CREATE INDEX idx_logs_task_id ON logs(task_id);

-- Function: Get tasks ready for execution (all dependencies completed)
CREATE OR REPLACE FUNCTION get_ready_tasks()
RETURNS TABLE (
    task_id UUID,
    description TEXT,
    created_at TIMESTAMPTZ
) AS $$
BEGIN
    RETURN QUERY
    SELECT t.id, t.description, t.created_at
    FROM tasks t
    WHERE t.status = 'pending'
      AND NOT EXISTS (
          SELECT 1
          FROM task_dependencies td
          JOIN tasks dep ON td.depends_on = dep.id
          WHERE td.task_id = t.id
            AND dep.status != 'completed'
      )
    ORDER BY t.created_at ASC;
END;
$$ LANGUAGE plpgsql;

-- Function: Atomically claim a task
CREATE OR REPLACE FUNCTION claim_task(p_task_id UUID, p_operator_id UUID)
RETURNS BOOLEAN AS $$
DECLARE
    v_success BOOLEAN;
BEGIN
    UPDATE tasks
    SET status = 'claimed',
        operator_id = p_operator_id,
        claimed_at = NOW()
    WHERE id = p_task_id
      AND status = 'pending'
    RETURNING TRUE INTO v_success;

    RETURN COALESCE(v_success, FALSE);
END;
$$ LANGUAGE plpgsql;

-- Function: Acquire an idle operator or create new one
CREATE OR REPLACE FUNCTION acquire_operator(p_operator_type operator_type)
RETURNS UUID AS $$
DECLARE
    v_operator_id UUID;
BEGIN
    -- Try to find an idle operator of the requested type
    SELECT id INTO v_operator_id
    FROM operators
    WHERE operator_type = p_operator_type
      AND status = 'idle'
    ORDER BY last_active_at ASC NULLS FIRST
    LIMIT 1
    FOR UPDATE SKIP LOCKED;

    -- If found, mark as active
    IF v_operator_id IS NOT NULL THEN
        UPDATE operators
        SET status = 'active',
            last_active_at = NOW()
        WHERE id = v_operator_id;

        RETURN v_operator_id;
    END IF;

    -- Otherwise, create a new operator
    INSERT INTO operators (operator_type, status, last_active_at)
    VALUES (p_operator_type, 'active', NOW())
    RETURNING id INTO v_operator_id;

    RETURN v_operator_id;
END;
$$ LANGUAGE plpgsql;

-- Function: Release operator back to pool
CREATE OR REPLACE FUNCTION release_operator(p_operator_id UUID, p_success BOOLEAN)
RETURNS VOID AS $$
BEGIN
    UPDATE operators
    SET status = CASE WHEN p_success THEN 'idle' ELSE 'failed' END,
        current_task_id = NULL,
        current_session_id = NULL,
        tasks_completed = CASE WHEN p_success THEN tasks_completed + 1 ELSE tasks_completed END,
        last_active_at = NOW()
    WHERE id = p_operator_id;
END;
$$ LANGUAGE plpgsql;

-- Function: Log an event
CREATE OR REPLACE FUNCTION log_event(
    p_level log_level,
    p_operator_id UUID,
    p_task_id UUID,
    p_message TEXT,
    p_metadata JSONB DEFAULT NULL
)
RETURNS UUID AS $$
DECLARE
    v_log_id UUID;
BEGIN
    INSERT INTO logs (level, operator_id, task_id, message, metadata)
    VALUES (p_level, p_operator_id, p_task_id, p_message, p_metadata)
    RETURNING id INTO v_log_id;

    RETURN v_log_id;
END;
$$ LANGUAGE plpgsql;
