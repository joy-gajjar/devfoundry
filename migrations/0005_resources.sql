CREATE TABLE IF NOT EXISTS resources (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL REFERENCES projects(id),
    version TEXT NOT NULL,
    source TEXT NOT NULL,
    manifest_hash TEXT NOT NULL,
    required_capabilities TEXT NOT NULL,
    revision INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(project_id, id)
);
CREATE TABLE IF NOT EXISTS resource_files (
    resource_id TEXT NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    project_id TEXT NOT NULL REFERENCES projects(id),
    target TEXT NOT NULL,
    expected_hash TEXT NOT NULL,
    installed_hash TEXT NOT NULL,
    size INTEGER NOT NULL,
    PRIMARY KEY(resource_id, target),
    UNIQUE(project_id, target)
);
CREATE INDEX IF NOT EXISTS idx_resources_project ON resources(project_id, updated_at);
