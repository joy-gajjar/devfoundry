CREATE TABLE IF NOT EXISTS runs (
    id TEXT PRIMARY KEY NOT NULL,
    session_id TEXT NOT NULL REFERENCES sessions(id),
    idempotency_key TEXT NOT NULL,
    prompt TEXT NOT NULL,
    expected_revision INTEGER NOT NULL,
    status TEXT NOT NULL,
    revision INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (session_id, idempotency_key)
);

CREATE TABLE IF NOT EXISTS attempts (
    id TEXT PRIMARY KEY NOT NULL,
    run_id TEXT NOT NULL REFERENCES runs(id),
    status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS execution_leases (
    run_id TEXT PRIMARY KEY NOT NULL REFERENCES runs(id),
    owner_id TEXT NOT NULL,
    revision INTEGER NOT NULL,
    expires_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tool_calls (
    id TEXT PRIMARY KEY NOT NULL,
    run_id TEXT NOT NULL REFERENCES runs(id),
    attempt_id TEXT NOT NULL REFERENCES attempts(id),
    name TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tool_outputs (
    tool_call_id TEXT PRIMARY KEY NOT NULL REFERENCES tool_calls(id),
    output TEXT NOT NULL,
    truncated INTEGER NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS run_events (
    run_id TEXT NOT NULL REFERENCES runs(id),
    sequence INTEGER NOT NULL,
    payload TEXT NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY (run_id, sequence)
);

CREATE INDEX IF NOT EXISTS idx_runs_session_status ON runs(session_id, status, updated_at);
CREATE INDEX IF NOT EXISTS idx_attempts_run_status ON attempts(run_id, status, updated_at);
CREATE INDEX IF NOT EXISTS idx_tool_calls_run_status ON tool_calls(run_id, status, updated_at);
CREATE INDEX IF NOT EXISTS idx_run_events_cursor ON run_events(run_id, sequence);
