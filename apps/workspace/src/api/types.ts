export type SessionStatus = 'idle' | 'running' | 'failed' | 'completed' | 'interrupted'
export type TaskStatus = 'draft' | 'ready' | 'assigned' | 'running' | 'blocked' | 'review' | 'accepted' | 'cancelled'

export interface Message {
  id: string
  session_id: string
  role: 'user' | 'assistant' | 'system' | 'tool'
  parts: Array<{ type: string; text?: string }>
  created_at: string
}

export interface Snapshot {
  version: 2
  session: {
    id: string
    project_id: string
    title: string
    agent: string
    model: { provider: string; name: string }
    status: SessionStatus
    created_at: string
    updated_at: string
  }
  messages: Message[]
  replay: { after: number; events_url: string }
}

export interface Task {
  id: string
  project_id: string
  title: string
  description: string
  status: TaskStatus
  revision: number
}

export interface Document {
  id: string
  project_id: string
  path: string
  title: string
  content_hash: string
  bytes: number
}

export interface DocumentListResponse {
  documents: Document[]
}

export interface HistoryResponse {
  version: 2
  messages: Message[]
  next: string | null
}

export interface WorkspaceError extends Error {
  kind: 'network' | 'http' | 'decode' | 'replay-gap'
  retryable: boolean
  status?: number
}

export interface BrowserBootstrap { version: 2; api_base: string }
export interface WorkspaceEvent { type: string; sequence?: number; [key: string]: unknown }

export interface TerminalInfo { terminal_id: string; project_id: string; platform: string }
export interface TerminalOutput { terminal_id: string; offset: number; next_offset: number; bytes: string; truncated: boolean; gap: boolean }
export interface WorkerProjection { id: string; task_id: string; status: string; revision?: number; evidence_count?: number }
export interface NotificationStatus { enabled: boolean; paired: boolean; revoked: boolean }
