import type { TerminalOutput } from '../api/types'

interface TerminalPanelProps {
  output?: TerminalOutput
  focused?: boolean
  onEscapeFocus?: () => void
}

export function TerminalPanel({ output, focused = false, onEscapeFocus }: TerminalPanelProps) {
  return <section className="terminal-panel" aria-label="Native terminal">
    <div className="terminal-panel-header"><strong>Terminal</strong><span role="status">{output?.gap ? 'Output gap; refresh required' : focused ? 'Focused (Ctrl-] to escape)' : 'Attached'}</span></div>
    <pre aria-live="polite">{output ? `[${output.offset}-${output.next_offset}] ${output.bytes}` : 'No terminal output.'}</pre>
    {output?.gap && <button type="button" onClick={onEscapeFocus}>Refresh terminal output</button>}
  </section>
}
