import { describe, expect, it } from 'vitest'
import { render, screen } from '@testing-library/react'
import { App } from '../src/App'

describe('workspace shell', () => {
  it('renders accessible navigation surfaces and a reconnect state', () => {
    render(<App initialState="reconnecting" />)

    expect(screen.getByRole('complementary', { name: 'Workspace navigation' })).toBeInTheDocument()
    expect(screen.getByRole('tab', { name: 'Chat' })).toBeInTheDocument()
    expect(screen.getByRole('tab', { name: 'Tasks' })).toBeInTheDocument()
    expect(screen.getByRole('tab', { name: 'Docs' })).toBeInTheDocument()
    expect(screen.getByRole('status')).toHaveTextContent('Reconnecting')
  })
})
