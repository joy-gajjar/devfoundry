CREATE TABLE IF NOT EXISTS scheduler_leases (
    id TEXT PRIMARY KEY NOT NULL,
    task_id TEXT NOT NULL REFERENCES tasks(id),
    task_revision INTEGER NOT NULL,
    owner TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    expires_at TEXT
);

CREATE TABLE IF NOT EXISTS worker_attempts (
    id TEXT PRIMARY KEY NOT NULL,
    lease_id TEXT NOT NULL REFERENCES scheduler_leases(id),
    task_id TEXT NOT NULL REFERENCES tasks(id),
    task_revision INTEGER NOT NULL,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (lease_id, id)
);

CREATE TABLE IF NOT EXISTS worker_evidence (
    id TEXT PRIMARY KEY NOT NULL,
    attempt_id TEXT NOT NULL REFERENCES worker_attempts(id),
    task_id TEXT NOT NULL REFERENCES tasks(id),
    task_revision INTEGER NOT NULL,
    base_commit TEXT NOT NULL,
    proposed_head TEXT NOT NULL,
    diff_digest TEXT NOT NULL,
    commands TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS failure_fingerprints (
    id TEXT PRIMARY KEY NOT NULL,
    task_id TEXT NOT NULL REFERENCES tasks(id),
    task_revision INTEGER NOT NULL,
    fingerprint TEXT NOT NULL,
    count INTEGER NOT NULL,
    last_seen_at TEXT NOT NULL,
    UNIQUE (task_id, task_revision, fingerprint)
);

CREATE TABLE IF NOT EXISTS integration_receipts (
    id TEXT PRIMARY KEY NOT NULL,
    task_id TEXT NOT NULL REFERENCES tasks(id),
    task_revision INTEGER NOT NULL,
    base_commit TEXT NOT NULL,
    resulting_commit TEXT NOT NULL,
    evidence_digest TEXT NOT NULL,
    created_at TEXT NOT NULL,
    UNIQUE (task_id, task_revision, resulting_commit)
);

CREATE TABLE IF NOT EXISTS scheduler_events (
    sequence INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id TEXT NOT NULL REFERENCES tasks(id),
    attempt_id TEXT,
    kind TEXT NOT NULL,
    payload TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_scheduler_active_lease
    ON scheduler_leases(task_id, task_revision) WHERE status = 'claimed';
CREATE INDEX IF NOT EXISTS idx_scheduler_leases_owner_status
    ON scheduler_leases(owner, status, updated_at);
CREATE INDEX IF NOT EXISTS idx_worker_attempts_task_status
    ON worker_attempts(task_id, status, updated_at);
CREATE INDEX IF NOT EXISTS idx_worker_attempts_lease
    ON worker_attempts(lease_id, created_at);
CREATE INDEX IF NOT EXISTS idx_worker_evidence_attempt
    ON worker_evidence(attempt_id, created_at);
CREATE INDEX IF NOT EXISTS idx_failure_fingerprints_task
    ON failure_fingerprints(task_id, task_revision, last_seen_at);
CREATE INDEX IF NOT EXISTS idx_integration_receipts_task
    ON integration_receipts(task_id, task_revision, created_at);
CREATE INDEX IF NOT EXISTS idx_scheduler_events_cursor
    ON scheduler_events(sequence, task_id);
