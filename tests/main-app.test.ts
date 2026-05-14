import { describe, it, expect, vi } from 'vitest';
import { render } from '@testing-library/svelte';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(async (cmd: string) => {
    if (cmd === 'get_config') return {
      language_a: 'en', language_b: 'vi', mine: 'b',
      hotkey: 'Ctrl+Shift+V', theme: 'system',
      default_meeting_source: null, default_mic: null,
    };
    if (cmd === 'has_api_key') return false;
    if (cmd === 'list_sessions') return [];
    if (cmd === 'list_mics') return [];
    if (cmd === 'list_loopback_sources') return [];
    return null;
  }),
}));
vi.mock('@tauri-apps/api/core', () => ({ invoke: invokeMock }));
vi.mock('@tauri-apps/api/event', () => ({ listen: async () => () => {} }));

import MainApp from '../src/routes/MainApp.svelte';

describe('MainApp', () => {
  it('mounts and shows the No-API-Key state when has_api_key returns false', async () => {
    const { findByText } = render(MainApp);
    const node = await findByText('Add your Soniox API key');
    expect(node).toBeTruthy();
  });
});
