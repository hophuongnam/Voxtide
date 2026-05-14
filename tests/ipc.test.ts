import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock `@tauri-apps/api/core` so unit tests don't need a real Tauri runtime.
// vi.hoisted ensures invokeMock is initialized before vi.mock hoisting runs.
const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
vi.mock('@tauri-apps/api/core', () => ({ invoke: invokeMock }));
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn() }));

import * as ipc from '../src/lib/ipc';

describe('ipc command wrappers', () => {
  beforeEach(() => invokeMock.mockReset());

  it('hasApiKey returns boolean', async () => {
    invokeMock.mockResolvedValueOnce(true);
    await expect(ipc.hasApiKey('alice')).resolves.toBe(true);
    expect(invokeMock).toHaveBeenCalledWith('has_api_key', { account: 'alice' });
  });

  it('setApiKey forwards both args', async () => {
    invokeMock.mockResolvedValueOnce(null);
    await ipc.setApiKey('alice', 'sk_test');
    expect(invokeMock).toHaveBeenCalledWith('set_api_key', { account: 'alice', secret: 'sk_test' });
  });

  it('startSession unwraps result number', async () => {
    invokeMock.mockResolvedValueOnce(42);
    const id = await ipc.startSession({
      mode: 'meeting',
      language_a: 'en', language_b: 'vi', mine: 'b',
      device_id: 'system', api_key_account: 'alice',
    });
    expect(id).toBe(42);
    expect(invokeMock).toHaveBeenCalledWith('start_session', expect.objectContaining({ req: expect.any(Object) }));
  });

  it('searchTranscripts passes the query', async () => {
    invokeMock.mockResolvedValueOnce([{ id: 1, session_id: 2, ts_ms: 0, text: 'hello' }]);
    const hits = await ipc.searchTranscripts('hello');
    expect(hits).toHaveLength(1);
    expect(invokeMock).toHaveBeenCalledWith('search_transcripts', { query: 'hello', limit: 50 });
  });
});
