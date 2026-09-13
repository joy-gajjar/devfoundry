import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { NotificationSettings } from './NotificationSettings'

describe('NotificationSettings', () => {
  it('makes disabled-by-default state explicit without rendering secrets', () => {
    render(<NotificationSettings status={{ enabled: false, paired: false, revoked: false }} onSetup={() => undefined} onRevoke={() => undefined} />)

    expect(screen.getByRole('heading', { name: 'Notifications' })).toBeTruthy()
    expect(screen.getByText('Notifications are disabled')).toBeTruthy()
    expect(screen.queryByText(/token|secret|command/i)).toBeNull()
  })
})
