import { useState } from 'react'

export type PreviewStatus = 'requested' | 'starting' | 'ready' | 'failed' | 'stopped'

export interface PreviewPanelProps {
  status: PreviewStatus
  previewUrl?: string
  error?: string
  onStop?: () => void
}

export function PreviewPanel({ status, previewUrl, error, onStop }: PreviewPanelProps) {
  const [annotation, setAnnotation] = useState('')
  const ready = status === 'ready' && Boolean(previewUrl)

  return <section aria-label="Preview" className="preview-panel">
    <div className="preview-heading"><div><span className="section-label">PREVIEW</span><h2>Isolated result</h2></div><span role="status" aria-label="Preview status">{status}</span></div>
    {ready ? <iframe title="Managed preview" src={previewUrl} sandbox="allow-scripts" /> : <p className="muted">{error ?? `Preview is ${status}.`}</p>}
    {onStop && status !== 'stopped' && <button type="button" onClick={onStop}>Stop preview</button>}
    <label htmlFor="preview-annotation">Annotation</label>
    <textarea id="preview-annotation" value={annotation} onChange={(event) => setAnnotation(event.target.value)} maxLength={2000} />
  </section>
}
