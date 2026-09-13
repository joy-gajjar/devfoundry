import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { PreviewPanel } from '../src/preview/PreviewPanel'

describe('preview panel', () => {
  it('keeps a non-ready preview out of the privileged document', () => {
    render(<PreviewPanel status="starting" previewUrl="http://127.0.0.1:4173" />)
    expect(screen.queryByTitle('Managed preview')).not.toBeInTheDocument()
    expect(screen.getByRole('status')).toHaveTextContent('starting')
  })
})
