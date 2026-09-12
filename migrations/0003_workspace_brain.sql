CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL REFERENCES projects(id),
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    status TEXT NOT NULL,
    revision INTEGER NOT NULL,
    session_id TEXT REFERENCES sessions(id),
    evidence_id TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS task_dependencies (
    task_id TEXT NOT NULL REFERENCES tasks(id),
    dependency_id TEXT NOT NULL REFERENCES tasks(id),
    PRIMARY KEY (task_id, dependency_id)
);

CREATE TABLE IF NOT EXISTS documents (
    project_id TEXT NOT NULL REFERENCES projects(id),
    path TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    links TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (project_id, path)
);

CREATE TABLE IF NOT EXISTS context_manifests (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL REFERENCES projects(id),
    payload TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_tasks_project_status ON tasks(project_id, status, updated_at);
CREATE INDEX IF NOT EXISTS idx_task_dependencies_dependency ON task_dependencies(dependency_id);
CREATE INDEX IF NOT EXISTS idx_documents_project ON documents(project_id, path);
