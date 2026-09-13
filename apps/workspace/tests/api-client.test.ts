import { describe, expect, it, vi } from 'vitest'
import { createWorkspaceClient } from '../src/api/client'
import { snapshotFixture } from '../src/api/fixtures'

describe('workspace API client', () => {
  it('loads a typed snapshot and preserves the replay cursor', async () => {
    const fetcher = vi.fn().mockResolvedValue(
      new Response(JSON.stringify(snapshotFixture), { status: 200 }),
    )
    const client = createWorkspaceClient({ baseUrl: 'http://localhost:3000', fetcher })

    const snapshot = await client.getSnapshot('session-1')

    expect(snapshot.version).toBe(2)
    expect(snapshot.replay.after).toBe(42)
    expect(fetcher).toHaveBeenCalledWith(
      'http://localhost:3000/api/v2/sessions/session-1/snapshot',
      expect.objectContaining({ method: 'GET' }),
    )
  })

  it('classifies a failed request as reconnectable without exposing response details', async () => {
    const fetcher = vi.fn().mockRejectedValue(new TypeError('network down'))
    const client = createWorkspaceClient({ baseUrl: 'http://localhost:3000', fetcher })

    await expect(client.getSnapshot('session-1')).rejects.toMatchObject({
      kind: 'network',
      retryable: true,
    })
  })

  it('never reads or persists an authorization token', async () => {
    const fetcher = vi.fn().mockResolvedValue(
      new Response(JSON.stringify(snapshotFixture), { status: 200 }),
    )
    const client = createWorkspaceClient({
      baseUrl: 'http://localhost:3000',
      fetcher,
      token: 'must-not-be-used',
    } as never)

    await client.getSnapshot('session-1')

    expect(fetcher.mock.calls[0][1]).not.toHaveProperty('headers.Authorization')
    expect(window.localStorage.length).toBe(0)
  })

  it('reconnects from the last durable cursor and refreshes after a replay gap', async () => {
    const fetcher = vi.fn()
      .mockResolvedValueOnce(new Response(JSON.stringify(snapshotFixture), { status: 200 }))
      .mockResolvedValueOnce(new Response('', { status: 409 }))
      .mockResolvedValueOnce(new Response(JSON.stringify(snapshotFixture), { status: 200 }))
    const client = createWorkspaceClient({ baseUrl: 'http://localhost:3000', fetcher })

    const stream = await client.reconcileSession('session-1', { replayGap: true })

    expect(stream.snapshot.replay.after).toBe(42)
    expect(fetcher).toHaveBeenNthCalledWith(3,
      'http://localhost:3000/api/v2/sessions/session-1/snapshot',
      expect.objectContaining({ method: 'GET' }),
    )
  })

  it('ignores unknown event kinds at the browser boundary', () => {
    const client = createWorkspaceClient()
    expect(client.acceptEvent({ type: 'future_event', sequence: 43 })).toBe(false)
  })
})
