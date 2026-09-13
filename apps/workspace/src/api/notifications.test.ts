import { describe, expect, it, vi } from 'vitest'
import { createWorkspaceClient } from './client'

describe('notification settings API', () => {
  it('reads disabled status and never stores credentials', async () => {
    const fetcher = vi.fn().mockResolvedValue(new Response(JSON.stringify({ enabled: false, paired: false, revoked: false }), { status: 200 }))
    const client = createWorkspaceClient({ fetcher })

    await expect(client.getNotificationStatus('project-1')).resolves.toEqual({ enabled: false, paired: false, revoked: false })
    expect(fetcher).toHaveBeenCalledWith('/api/v2/projects/project-1/notifications/status', expect.objectContaining({ credentials: 'same-origin' }))
    expect(localStorage.length).toBe(0)
  })
})
