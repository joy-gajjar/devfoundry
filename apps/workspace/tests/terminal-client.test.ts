import { describe, expect, it, vi } from 'vitest'
import { createWorkspaceClient } from '../src/api/client'

describe('workspace terminal projection', () => {
  it('replays bounded output from an explicit offset and preserves gap metadata', async () => {
    const fetcher = vi.fn().mockResolvedValue(new Response(JSON.stringify({
      terminal_id: 'terminal-1',
      offset: 12,
      next_offset: 18,
      bytes: 'b3V0cHV0',
      truncated: true,
      gap: false,
    }), { status: 200 }))
    const client = createWorkspaceClient({ baseUrl: 'http://localhost:3000', fetcher })

    const output = await client.getTerminalOutput('terminal-1', 12)

    expect(output.next_offset).toBe(18)
    expect(output.truncated).toBe(true)
    expect(fetcher).toHaveBeenCalledWith(
      'http://localhost:3000/api/v2/terminals/terminal-1/output?after=12',
      expect.objectContaining({ method: 'GET' }),
    )
  })
})
