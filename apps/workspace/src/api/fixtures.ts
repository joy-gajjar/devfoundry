import type { DocumentListResponse, HistoryResponse, Snapshot, Task } from './types'

export const snapshotFixture: Snapshot = {
  version: 2,
  session: {
    id: 'session-1',
    project_id: 'project-1',
    title: 'Workspace migration',
    agent: 'copilot',
    model: { provider: 'github-copilot', name: 'fixture-model' },
    status: 'running',
    created_at: '2026-09-13T10:00:00Z',
    updated_at: '2026-09-13T10:01:00Z',
  },
  messages: [
    { id: 'message-1', session_id: 'session-1', role: 'user', parts: [{ type: 'text', text: 'Review the workspace plan.' }], created_at: '2026-09-13T10:00:01Z' },
    { id: 'message-2', session_id: 'session-1', role: 'assistant', parts: [{ type: 'text', text: 'I am checking the bounded changes.' }], created_at: '2026-09-13T10:00:05Z' },
  ],
  replay: { after: 42, events_url: '/api/v2/sessions/session-1/events' },
}

export const historyFixture: HistoryResponse = { version: 2, messages: snapshotFixture.messages, next: null }
export const tasksFixture: Task[] = [
  { id: 'task-1', project_id: 'project-1', title: 'Review API contract', description: 'Check the v2 replay boundary.', status: 'review', revision: 3 },
  { id: 'task-2', project_id: 'project-1', title: 'Connect docs map', description: 'Expose linked project notes.', status: 'ready', revision: 1 },
]
export const documentsFixture: DocumentListResponse = {
  documents: [
    { id: 'doc-1', project_id: 'project-1', path: 'docs/README.md', title: 'Project README', content_hash: 'sha256:fixture', bytes: 2048 },
    { id: 'doc-2', project_id: 'project-1', path: 'docs/roadmap.md', title: 'Roadmap', content_hash: 'sha256:roadmap', bytes: 4096 },
  ],
}
