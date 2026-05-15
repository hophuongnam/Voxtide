import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';

const sampleSessions = [
  { id: 1, started_at: Date.now() - 60_000, ended_at: Date.now() - 30_000,
    mode: 'meeting', lang_a: 'en', lang_b: 'vi', device_label: 'Zoom',
    duration_ms: 30_000 },
  { id: 2, started_at: Date.now() - 90 * 60_000, ended_at: Date.now() - 88 * 60_000,
    mode: 'conversation', lang_a: 'en', lang_b: 'ja', device_label: null,
    duration_ms: 120_000 },
];

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(async (cmd: string, _args?: any) => {
    if (cmd === 'get_config') return {
      language_a: 'en', language_b: 'vi', mine: 'b',
      hotkey: 'Ctrl+Shift+V', theme: 'system',
      default_meeting_source: null, default_mic: null,
      mode: 'meeting', font_size: 'm', show_pinyin: false,
    };
    if (cmd === 'has_api_key') return true;
    if (cmd === 'list_sessions') return sampleSessions;
    if (cmd === 'list_mics') return [];
    if (cmd === 'list_loopback_sources') return [];
    if (cmd === 'delete_session') return null;
    return null;
  }),
}));
vi.mock('@tauri-apps/api/core', () => ({ invoke: invokeMock }));
vi.mock('@tauri-apps/api/event', () => ({ listen: async () => () => {} }));

import MainApp from '../src/routes/MainApp.svelte';

describe('MainApp delete flow', () => {
  it('hover trash → confirm modal → invokes delete_session and refreshes list', async () => {
    invokeMock.mockClear();
    const { container, findByText, getByText } = render(MainApp);

    // Wait until the sidebar renders both sessions.
    await waitFor(() => {
      expect(container.querySelectorAll('[data-testid="delete-session"]').length).toBe(2);
    });

    const trashButtons = container.querySelectorAll('[data-testid="delete-session"]');
    await fireEvent.click(trashButtons[0]!);

    // Modal appears with the canonical body line.
    await findByText(/Delete this transcript\?/i);
    await findByText(/cannot be undone/i);

    // Confirming fires the IPC command.
    await fireEvent.click(getByText('Delete'));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith('delete_session', { id: sampleSessions[0]!.id });
    });

    // After success the list is re-fetched (second list_sessions call).
    const listCalls = invokeMock.mock.calls.filter(([c]) => c === 'list_sessions');
    expect(listCalls.length).toBeGreaterThanOrEqual(2);
  });

  it('Cancel dismisses the modal without invoking delete_session', async () => {
    invokeMock.mockClear();
    const { container, getByText, queryByText } = render(MainApp);
    await waitFor(() => {
      expect(container.querySelectorAll('[data-testid="delete-session"]').length).toBe(2);
    });
    await fireEvent.click(container.querySelectorAll('[data-testid="delete-session"]')[0]!);
    await fireEvent.click(getByText('Cancel'));
    await waitFor(() => {
      expect(queryByText(/Delete this transcript\?/i)).toBeNull();
    });
    expect(invokeMock).not.toHaveBeenCalledWith('delete_session', expect.anything());
  });

  it('shows error inline when delete_session rejects', async () => {
    invokeMock.mockClear();
    invokeMock.mockImplementation(async (cmd: string) => {
      if (cmd === 'get_config') return {
        language_a: 'en', language_b: 'vi', mine: 'b',
        hotkey: 'Ctrl+Shift+V', theme: 'system',
        default_meeting_source: null, default_mic: null,
        mode: 'meeting', font_size: 'm', show_pinyin: false,
      };
      if (cmd === 'has_api_key') return true;
      if (cmd === 'list_sessions') return sampleSessions;
      if (cmd === 'list_mics' || cmd === 'list_loopback_sources') return [];
      if (cmd === 'delete_session') throw new Error('cannot delete an active session');
      return null;
    });
    const { container, getByText, findByText } = render(MainApp);
    await waitFor(() => {
      expect(container.querySelectorAll('[data-testid="delete-session"]').length).toBe(2);
    });
    await fireEvent.click(container.querySelectorAll('[data-testid="delete-session"]')[0]!);
    await fireEvent.click(getByText('Delete'));
    await findByText(/cannot delete an active session/i);
    // Modal stays open.
    expect(getByText(/Delete this transcript\?/i)).toBeTruthy();
  });

  it('mounts and shows the No-API-Key state when has_api_key returns false', async () => {
    invokeMock.mockClear();
    (invokeMock as any).mockImplementation(async (cmd: string) => {
      if (cmd === 'get_config') return {
        language_a: 'en', language_b: 'vi', mine: 'b',
        hotkey: 'Ctrl+Shift+V', theme: 'system',
        default_meeting_source: null, default_mic: null,
        mode: 'meeting', font_size: 'm', show_pinyin: false,
      };
      if (cmd === 'has_api_key') return false;
      if (cmd === 'list_sessions') return [];
      if (cmd === 'list_mics' || cmd === 'list_loopback_sources') return [];
      return null;
    });
    const { findByText } = render(MainApp);
    const node = await findByText('Add your Soniox API key');
    expect(node).toBeTruthy();
  });
});

describe('MainApp reading config', () => {
  afterEach(async () => {
    const { transcript } = await import('../src/lib/stores.svelte');
    transcript.reset();
  });

  it('passes show_pinyin through to a live zh transcript line', async () => {
    invokeMock.mockClear();
    invokeMock.mockImplementation(async (cmd: string) => {
      if (cmd === 'get_config') return {
        language_a: 'zh', language_b: 'en', mine: 'a',
        hotkey: 'Ctrl+Shift+V', theme: 'system',
        default_meeting_source: null, default_mic: null,
        mode: 'conversation', font_size: 'm', show_pinyin: true,
      };
      if (cmd === 'has_api_key') return true;
      if (cmd === 'list_sessions') return [];
      if (cmd === 'list_mics' || cmd === 'list_loopback_sources') return [];
      return null;
    });

    const { transcript } = await import('../src/lib/stores.svelte');
    transcript.reset();   // store is a module singleton — isolate from other tests

    const { container } = render(MainApp);

    // Drive a final zh line through the transcript store the same way
    // handleCoreEvent would, then assert RubyText rendered it.
    transcript.final({ status: 'original', text: '你好', chip: null, language: 'zh', ts_ms: 1 });

    await waitFor(() => {
      expect(container.querySelectorAll('ruby').length).toBe(2);
    });
  });

  it('passes font_size through to the transcript-root CSS variable', async () => {
    invokeMock.mockClear();
    invokeMock.mockImplementation(async (cmd: string) => {
      if (cmd === 'get_config') return {
        language_a: 'zh', language_b: 'en', mine: 'a',
        hotkey: 'Ctrl+Shift+V', theme: 'system',
        default_meeting_source: null, default_mic: null,
        mode: 'conversation', font_size: 'xl', show_pinyin: false,
      };
      if (cmd === 'has_api_key') return true;
      if (cmd === 'list_sessions') return [];
      if (cmd === 'list_mics' || cmd === 'list_loopback_sources') return [];
      return null;
    });

    const { transcript } = await import('../src/lib/stores.svelte');
    transcript.reset();
    transcript.final({ status: 'original', text: 'hi', chip: null, language: 'en', ts_ms: 1 });

    const { container } = render(MainApp);

    await waitFor(() => {
      const root = container.querySelector('[data-testid="transcript-root"]') as HTMLElement;
      expect(root).not.toBeNull();
      expect(root.style.getPropertyValue('--vt-transcript-size')).toBe('19px');
    });
  });
});
