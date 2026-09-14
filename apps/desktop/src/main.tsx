import { StrictMode, useEffect, useState } from 'react'
import { createRoot } from 'react-dom/client'
import './styles.css'

type BootState = 'starting' | 'ready' | 'error'

function App() {
  const [state, setState] = useState<BootState>('starting')
  const [message, setMessage] = useState('Starting the local DevFoundry host...')

  useEffect(() => {
    const timer = window.setTimeout(() => {
      setState('ready')
      setMessage('The desktop shell is ready. The workspace UI will load in the next phase.')
    }, 150)
    return () => window.clearTimeout(timer)
  }, [])

  return (
    <main className="boot-shell" aria-labelledby="title">
      <div className={`boot-mark ${state}`} aria-hidden="true">DF</div>
      <p className="eyebrow">DEVFOUNDRY / TAURI DESKTOP</p>
      <h1 id="title">{state === 'ready' ? 'Workspace ready' : 'Starting workspace'}</h1>
      <p role="status">{message}</p>
      {state === 'error' && <button type="button" onClick={() => window.location.reload()}>Retry</button>}
    </main>
  )
}

createRoot(document.getElementById('root')!).render(<StrictMode><App /></StrictMode>)
