import { useEffect, useState } from 'react'
import { documentsFixture, snapshotFixture, tasksFixture } from './api/fixtures'
import './styles.css'
import { PreviewPanel } from './preview/PreviewPanel'
import { TerminalPanel } from './terminal/TerminalPanel'
import { NotificationSettings } from './notifications/NotificationSettings'

type Surface = 'Chat' | 'Tasks' | 'Docs' | 'Preview' | 'Terminal' | 'Settings'
type AppState = 'ready' | 'reconnecting' | 'error'
type Theme = 'dark' | 'light'

interface AppProps { initialState?: AppState }

const surfaceCopy: Record<Surface, string> = {
  Chat: 'Session transcript',
  Tasks: 'Task board',
  Docs: 'Project documents',
  Preview: 'Managed preview',
  Terminal: 'Native terminal',
  Settings: 'Notification settings',
}

export function App({ initialState = 'ready' }: AppProps) {
  const [surface, setSurface] = useState<Surface>('Chat')
  const [state] = useState(initialState)
  const [theme, setTheme] = useState<Theme>(() => (localStorage.getItem('devfoundry-theme') as Theme) || 'dark')
  const activeSurface = surfaceCopy[surface]

  useEffect(() => {
    document.documentElement.dataset.theme = theme
    localStorage.setItem('devfoundry-theme', theme)
  }, [theme])

  return (
    <div className="workspace-shell">
      <a className="skip-link" href="#main-content">Skip to workspace</a>
      <header className="topbar">
        <div>
          <p className="eyebrow">DEVFOUNDRY / LOCAL WORKSPACE</p>
          <h1>Workspace</h1>
        </div>
        <div className="topbar-actions">
          <button className="theme-toggle" type="button" onClick={() => setTheme(theme === 'dark' ? 'light' : 'dark')} aria-label={`Switch to ${theme === 'dark' ? 'light' : 'dark'} theme`}>
            {theme === 'dark' ? 'Light mode' : 'Dark mode'}
          </button>
          <div className="connection-cluster">
          <span className={`connection-dot ${state}`} aria-hidden="true" />
          <span role="status">{state === 'reconnecting' ? 'Reconnecting' : state === 'error' ? 'API unavailable' : 'Connected'}</span>
          </div>
        </div>
      </header>

      <div className="workspace-grid">
        <aside className="sidebar" aria-label="Workspace navigation">
          <section className="project-card">
            <span className="section-label">PROJECT</span>
            <strong>OpenCode Rust</strong>
            <span className="muted">local / project-1</span>
          </section>
          <nav className="session-nav" aria-label="Sessions">
            <span className="section-label">SESSION</span>
            <button className="session-button active" type="button">Workspace migration<span>running</span></button>
            <button className="session-button" type="button">API contract review<span>idle</span></button>
          </nav>
          <div className="sidebar-footer"><span className="section-label">MODEL</span><strong>Copilot fixture</strong><span className="muted">No credentials in browser</span></div>
        </aside>

        <main className="main-panel" id="main-content" tabIndex={-1}>
          <div className="mobile-tabs" role="tablist" aria-label="Workspace surfaces">
              {(['Chat', 'Tasks', 'Docs', 'Preview', 'Terminal', 'Settings'] as Surface[]).map((item) => (
              <button key={item} role="tab" aria-selected={surface === item} aria-controls="surface-content" className={surface === item ? 'selected' : ''} type="button" onClick={() => setSurface(item)}>{item}</button>
            ))}
          </div>
          <div className="surface-heading"><div><span className="section-label">{surface}</span><h2>{activeSurface}</h2></div><span className="revision">snapshot #42</span></div>
          {state !== 'ready' && <div className="alert" role="alert"><strong>{state === 'reconnecting' ? 'Live updates paused.' : 'The API is unavailable.'}</strong><span>{state === 'reconnecting' ? 'The last durable snapshot remains visible while reconnecting.' : 'Refresh or check the local DevFoundry host.'}</span></div>}
          <div id="surface-content">
          {surface === 'Chat' && <ChatSurface />}
          {surface === 'Tasks' && <TaskSurface />}
           {surface === 'Docs' && <DocsSurface />}
            {surface === 'Preview' && <PreviewPanel status="ready" previewUrl="http://127.0.0.1:4173" onStop={() => undefined} />}
           {surface === 'Terminal' && <TerminalPanel />}
           {surface === 'Settings' && <NotificationSettings status={{ enabled: false, paired: false, revoked: false }} onSetup={() => undefined} onRevoke={() => undefined} />}
          </div>
        </main>
        <aside className="inspector" aria-label="Workspace status">
          <section><span className="section-label">RUN STATUS</span><div className="status-line"><span className="status-chip running">RUNNING</span><span>1 active session</span></div></section>
          <section><span className="section-label">USAGE</span><strong className="usage-number">42</strong><span className="muted">events replayed</span></section>
          <section><span className="section-label">SAFETY</span><p className="muted">Browser display is value-free. Approval and credential actions remain host-controlled.</p></section>
        </aside>
      </div>
    </div>
  )
}

function ChatSurface() {
  return <div className="chat-layout"><div className="transcript" aria-label="Session transcript">{snapshotFixture.messages.map((message) => <article className={`message ${message.role}`} key={message.id}><div className="message-meta"><strong>{message.role === 'assistant' ? 'Copilot' : 'You'}</strong><time>{message.created_at.slice(11, 16)}</time></div><p>{message.parts.find((part) => part.type === 'text')?.text}</p></article>)}</div><form className="composer" onSubmit={(event) => event.preventDefault()}><label htmlFor="message">Message the session</label><textarea id="message" placeholder="Describe the next bounded step..." /><button type="submit">Send prompt</button><small>Admission is routed through the authenticated host. Nothing is stored in this browser.</small></form></div>
}

function TaskSurface() {
  return <div className="card-list">{tasksFixture.map((task) => <article className="task-card" key={task.id}><div><span className={`status-chip ${task.status}`}>{task.status}</span><h3>{task.title}</h3><p>{task.description}</p></div><button type="button">Open session</button></article>)}</div>
}

function DocsSurface() {
  return <div className="card-list">{documentsFixture.documents.map((document) => <article className="document-card" key={document.id}><div className="doc-icon" aria-hidden="true">MD</div><div><h3>{document.title}</h3><p>{document.path}</p></div><span className="muted">{document.bytes} B</span></article>)}</div>
}
