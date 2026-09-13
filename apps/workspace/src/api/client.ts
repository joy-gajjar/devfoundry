import type { BrowserBootstrap, DocumentListResponse, HistoryResponse, Snapshot, Task, TerminalOutput, WorkerProjection, WorkspaceError, WorkspaceEvent } from './types'

type Fetcher = typeof fetch

interface ClientOptions {
  baseUrl?: string
  fetcher?: Fetcher
}

function workspaceError(kind: WorkspaceError['kind'], message: string, retryable: boolean, status?: number): WorkspaceError {
  const error = new Error(message) as WorkspaceError
  error.kind = kind
  error.retryable = retryable
  error.status = status
  return error
}

export function createWorkspaceClient(options: ClientOptions = {}) {
  const baseUrl = (options.baseUrl ?? '').replace(/\/$/, '')
  const fetcher = options.fetcher ?? fetch

  async function get<T>(path: string): Promise<T> {
    let response: Response
    try {
      response = await fetcher(`${baseUrl}${path}`, { method: 'GET', credentials: 'same-origin', headers: { Accept: 'application/json' } })
    } catch {
      throw workspaceError('network', 'The workspace API could not be reached.', true)
    }
    if (!response.ok) {
      throw workspaceError('http', 'The workspace API returned an error.', response.status >= 500 || response.status === 429, response.status)
    }
    try {
      return await response.json() as T
    } catch {
      throw workspaceError('decode', 'The workspace response was invalid.', false, response.status)
    }
  }

  async function reconcileSession(sessionId: string, options: { replayGap?: boolean } = {}) {
    const snapshot = await get<Snapshot>(`/api/v2/sessions/${encodeURIComponent(sessionId)}/snapshot`)
    if (options.replayGap) {
      try {
        await get<unknown>(`${snapshot.replay.events_url}?after=${snapshot.replay.after}`)
      } catch {
        return {
          snapshot: await get<Snapshot>(`/api/v2/sessions/${encodeURIComponent(sessionId)}/snapshot`),
          cursor: snapshot.replay.after,
        }
      }
    }
    return { snapshot, cursor: snapshot.replay.after }
  }

  function acceptEvent(event: WorkspaceEvent): boolean {
    return ['session_status', 'message_created', 'message_updated', 'permission_requested', 'permission_resolved'].includes(event.type)
  }

  return {
    getSnapshot: (sessionId: string) => get<Snapshot>(`/api/v2/sessions/${encodeURIComponent(sessionId)}/snapshot`),
    getHistory: (sessionId: string, after?: string) => get<HistoryResponse>(`/api/v2/sessions/${encodeURIComponent(sessionId)}/messages?limit=100${after ? `&after=${encodeURIComponent(after)}` : ''}`),
    getTasks: (workspaceId: string) => get<Task[]>(`/api/v2/workspaces/${encodeURIComponent(workspaceId)}/tasks`),
    getDocuments: (projectId: string) => get<DocumentListResponse>(`/api/v2/projects/${encodeURIComponent(projectId)}/documents`),
    getBootstrap: () => get<BrowserBootstrap>('/api/v2/browser/bootstrap'),
    getTerminalOutput: (terminalId: string, after = 0) => get<TerminalOutput>(`/api/v2/terminals/${encodeURIComponent(terminalId)}/output?after=${after}`),
    getWorkers: (workspaceId: string) => get<WorkerProjection[]>(`/api/v2/workspaces/${encodeURIComponent(workspaceId)}/workers`),
    reconcileSession,
    acceptEvent,
  }
}
