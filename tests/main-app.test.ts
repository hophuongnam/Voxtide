import { describe, it, expect, vi, afterEach, beforeEach } from 'vitest';
import { render, fireEvent, waitFor, within, cleanup } from '@testing-library/svelte';

const sampleSessions = [
  { id: 1, started_at: Date.now() - 60_000, ended_at: Date.now() - 30_000,
    mode: 'meeting', lang_a: 'en', lang_b: 'vi', device_label: 'Zoom',
    duration_ms: 30_000 },
  { id: 2, started_at: Date.now() - 90 * 60_000, ended_at: Date.now() - 88 * 60_000,
    mode: 'conversation', lang_a: 'en', lang_b: 'ja', device_label: null,
    duration_ms: 120_000 },
];

const { invokeMock } = vi.hoisted(() => ({
  // Return type is widened to `unknown` so per-test mockImplementations can
  // return any command's payload shape (device lists, structured errors, …)
  // without fighting the union inferred from this default body.
  invokeMock: vi.fn(async (cmd: string, _args?: any): Promise<unknown> => {
    if (cmd === 'get_config') return {
      language_a: 'en', language_b: 'vi',
      hotkey: 'Ctrl+Shift+V', theme: 'system',
      default_meeting_source: null, default_mic: null, meeting_capture_mic: false,
      mode: 'meeting', font_size: 'm', show_pinyin: false,
      context: '', contexts: [], active_context_id: null,
    };
    if (cmd === 'has_api_key') return true;
    if (cmd === 'list_sessions') return sampleSessions;
    if (cmd === 'list_mics') return [];
    if (cmd === 'list_loopback_sources') return [];
    if (cmd === 'delete_session') return null;
    if (cmd === 'app_info') return { model: 'stt-rt-v5', sample_rate_hz: 16000, channels: 1 };
    return null;
  }),
}));
vi.mock('@tauri-apps/api/core', () => ({ invoke: invokeMock }));

// Capture event listeners by name so tests can dispatch a backend event
// (e.g. a `voxtide://event` core event) into the live handler. Existing tests
// that never emit are unaffected — `listen` still resolves to an unlisten fn.
const { listeners, emitEvent } = vi.hoisted(() => {
  const listeners = new Map<string, (e: { payload: unknown }) => void>();
  return {
    listeners,
    emitEvent(name: string, payload: unknown) {
      listeners.get(name)?.({ payload });
    },
  };
});
vi.mock('@tauri-apps/api/event', () => ({
  listen: async (name: string, handler: (e: { payload: unknown }) => void) => {
    listeners.set(name, handler);
    return () => listeners.delete(name);
  },
}));

import MainApp from '../src/routes/MainApp.svelte';

// Wait until the toolbar's source picker (scoped to this instance) shows the
// given selected device label — i.e. MainApp's onMount has populated the device
// list AND the reactive $effect has assigned `selectedSource`. This is the exact
// precondition for onStart to proceed past its guard; gating on it removes the
// race between the async onMount and a click, which otherwise no-ops silently.
async function waitForStartReady(container: HTMLElement, sourceLabel: string) {
  await waitFor(() => {
    const labels = [...container.querySelectorAll('span')].map(s => s.textContent?.trim());
    expect(labels).toContain(sourceLabel);
  }, { timeout: 2000 });
}

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
        language_a: 'en', language_b: 'vi',
        hotkey: 'Ctrl+Shift+V', theme: 'system',
        default_meeting_source: null, default_mic: null, meeting_capture_mic: false,
        mode: 'meeting', font_size: 'm', show_pinyin: false,
        context: '', contexts: [], active_context_id: null,
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
        language_a: 'en', language_b: 'vi',
        hotkey: 'Ctrl+Shift+V', theme: 'system',
        default_meeting_source: null, default_mic: null, meeting_capture_mic: false,
        mode: 'meeting', font_size: 'm', show_pinyin: false,
        context: '', contexts: [], active_context_id: null,
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
        language_a: 'zh', language_b: 'en',
        hotkey: 'Ctrl+Shift+V', theme: 'system',
        default_meeting_source: null, default_mic: null, meeting_capture_mic: false,
        mode: 'conversation', font_size: 'm', show_pinyin: true,
        context: '', contexts: [], active_context_id: null,
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
        language_a: 'zh', language_b: 'en',
        hotkey: 'Ctrl+Shift+V', theme: 'system',
        default_meeting_source: null, default_mic: null, meeting_capture_mic: false,
        mode: 'conversation', font_size: 'xl', show_pinyin: false,
        context: '', contexts: [], active_context_id: null,
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

  it('past-viewer TranscriptPane receives fontSize and showPinyin from config', async () => {
    const pastSession = {
      id: 99, started_at: Date.now() - 120_000, ended_at: Date.now() - 60_000,
      mode: 'conversation', lang_a: 'zh', lang_b: 'en', device_label: null,
      duration_ms: 60_000,
    };
    const zhToken = {
      id: 1, session_id: 99, ts_ms: 1, text: '你好', language: 'zh',
      status: 'original', speaker: null,
    };

    invokeMock.mockClear();
    (invokeMock as any).mockImplementation(async (cmd: string) => {
      if (cmd === 'get_config') return {
        language_a: 'zh', language_b: 'en',
        hotkey: 'Ctrl+Shift+V', theme: 'system',
        default_meeting_source: null, default_mic: null, meeting_capture_mic: false,
        mode: 'conversation', font_size: 'xl', show_pinyin: true,
        context: '', contexts: [], active_context_id: null,
      };
      if (cmd === 'has_api_key') return true;
      if (cmd === 'list_sessions') return [pastSession];
      if (cmd === 'list_mics' || cmd === 'list_loopback_sources') return [];
      if (cmd === 'get_session') return { session: pastSession, tokens: [zhToken] };
      return null;
    });

    const { transcript } = await import('../src/lib/stores.svelte');
    transcript.reset();

    const { container } = render(MainApp);

    // Wait for the sidebar session item to appear, then click it to enter past-viewer.
    await waitFor(() => {
      expect(container.querySelectorAll('button[data-active]').length).toBeGreaterThan(0);
    });
    const sessionBtn = container.querySelector('button[data-active]') as HTMLElement;
    await fireEvent.click(sessionBtn);

    // After the get_session IPC resolves, past-viewer TranscriptPane should render
    // with --vt-transcript-size: 19px (font_size 'xl') and ruby elements (show_pinyin + zh token).
    await waitFor(() => {
      const root = container.querySelector('[data-testid="transcript-root"]') as HTMLElement;
      expect(root).not.toBeNull();
      expect(root.style.getPropertyValue('--vt-transcript-size')).toBe('19px');
      expect(container.querySelectorAll('ruby').length).toBeGreaterThan(0);
    });
  });
});

describe('MainApp error surfacing', () => {
  beforeEach(async () => {
    // Tear down any MainApp left mounted by a prior test whose async onMount
    // outran auto-cleanup; otherwise a stale instance's source picker satisfies
    // global text queries while THIS instance is still settling.
    cleanup();
    // Clear stale event listeners from a prior test's pending onMount — without
    // this, a handler registered by an earlier instance can clobber the Map
    // entry that the current instance is about to register.
    listeners.clear();
    // Stores are module singletons — isolate from other tests so the
    // EmptyState (gated on no live transcript) renders deterministically.
    const { transcript, session } = await import('../src/lib/stores.svelte');
    transcript.reset();
    session.stop();
  });

  const baseConfig = {
    language_a: 'en', language_b: 'vi',
    hotkey: 'Ctrl+Shift+V', theme: 'system',
    default_meeting_source: null, default_mic: null, meeting_capture_mic: false,
    mode: 'meeting', font_size: 'm', show_pinyin: false,
    context: '', contexts: [], active_context_id: null,
  };
  const loopback = [{ id: 'sys', label: 'System Audio', default: true }];

  it('renders a provider error core event in the error strip', async () => {
    invokeMock.mockClear();
    invokeMock.mockImplementation(async (cmd: string) => {
      if (cmd === 'get_config') return baseConfig;
      if (cmd === 'has_api_key') return true;
      if (cmd === 'list_sessions') return [];
      if (cmd === 'list_mics') return [];
      if (cmd === 'list_loopback_sources') return [];
      return null;
    });

    const { container } = render(MainApp);

    // Wait until the core-event listener is registered, then dispatch an error.
    await waitFor(() => {
      expect(listeners.has('voxtide://event')).toBe(true);
    });
    emitEvent('voxtide://event', { kind: 'error', message: 'Soniox error 401: bad key' });

    const strip = await waitFor(() => {
      const el = container.querySelector('[data-testid="app-error"]');
      expect(el).not.toBeNull();
      return el!;
    });
    expect(strip.textContent).toMatch(/Soniox error 401: bad key/i);
  });

  it('start_session device-missing rejection shows the strip, not the permission banner', async () => {
    invokeMock.mockClear();
    invokeMock.mockImplementation(async (cmd: string) => {
      if (cmd === 'get_config') return baseConfig;
      if (cmd === 'has_api_key') return true;
      if (cmd === 'list_sessions') return [];
      if (cmd === 'list_mics') return [];
      if (cmd === 'list_loopback_sources') return loopback;
      if (cmd === 'start_session') {
        // Tauri rejects with the structured StartError payload.
        throw { kind: 'device-missing', message: 'mic device not found: USB Mic' };
      }
      return null;
    });

    const { container } = render(MainApp);
    const scoped = within(container);

    // Wait until this instance's onMount fully completes (refreshSources fetches
    // mics last) and the source picker shows the selected device — only then is
    // onStart's `hasApiKey && config && selectedSource` guard satisfied.
    await waitForStartReady(container, 'System Audio');
    await fireEvent.click(scoped.getByRole('button', { name: /Start/ }));

    // The error strip appears carrying the raw device-missing message.
    const strip = await waitFor(() => {
      const el = container.querySelector('[data-testid="app-error"]');
      expect(el).not.toBeNull();
      return el!;
    });
    expect(strip.textContent).toMatch(/mic device not found: USB Mic/i);
    // The permission banner must NOT appear for a device-missing error.
    expect(container.querySelector('[data-testid="permission-banner"]')).toBeNull();
  });

  it('start_session mic-permission rejection shows the permission banner (not the strip)', async () => {
    invokeMock.mockClear();
    invokeMock.mockImplementation(async (cmd: string) => {
      if (cmd === 'get_config') return { ...baseConfig, mode: 'conversation' };
      if (cmd === 'has_api_key') return true;
      if (cmd === 'list_sessions') return [];
      if (cmd === 'list_mics') return [{ id: 'builtin', label: 'Built-in Mic', default: true }];
      if (cmd === 'list_loopback_sources') return [];
      if (cmd === 'start_session') {
        throw { kind: 'mic-permission', message: 'audio: cpal build_input_stream: denied' };
      }
      return null;
    });

    const { container } = render(MainApp);
    const scoped = within(container);

    await waitForStartReady(container, 'Built-in Mic');
    await fireEvent.click(scoped.getByRole('button', { name: /Start/ }));

    await waitFor(() => {
      expect(container.querySelector('[data-testid="permission-banner"]')).not.toBeNull();
    });
    // The plain error strip must NOT appear — this is a routed permission case.
    expect(container.querySelector('[data-testid="app-error"]')).toBeNull();
  });
});

describe('MainApp language swap', () => {
  it('swap button exchanges source/target languages and persists immediately', async () => {
    invokeMock.mockClear();
    invokeMock.mockImplementation(async (cmd: string) => {
      if (cmd === 'get_config') return {
        language_a: 'en', language_b: 'vi',
        hotkey: 'Ctrl+Shift+V', theme: 'system',
        default_meeting_source: null, default_mic: null, meeting_capture_mic: false,
        mode: 'meeting', font_size: 'm', show_pinyin: false,
        context: '', contexts: [], active_context_id: null,
      };
      if (cmd === 'has_api_key') return true;
      if (cmd === 'list_sessions') return [];
      if (cmd === 'list_mics' || cmd === 'list_loopback_sources') return [];
      return null;
    });

    const { container } = render(MainApp);
    const swapBtn = await waitFor(() => {
      const b = container.querySelector('button[aria-label="Swap languages"]') as HTMLElement;
      expect(b).not.toBeNull();
      return b;
    });
    await fireEvent.click(swapBtn);

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith('set_config', expect.objectContaining({
        cfg: expect.objectContaining({ language_a: 'vi', language_b: 'en' }),
      }));
    });
  });
});

describe('MainApp search', () => {
  const bootMock = (extra?: (cmd: string, args?: any) => unknown | undefined) =>
    (invokeMock as any).mockImplementation(async (cmd: string, args?: any) => {
      const handled = extra?.(cmd, args);
      if (handled !== undefined) return handled;
      if (cmd === 'get_config') return {
        language_a: 'en', language_b: 'vi',
        hotkey: 'Ctrl+Shift+V', theme: 'system',
        default_meeting_source: null, default_mic: null, meeting_capture_mic: false,
        mode: 'meeting', font_size: 'm', show_pinyin: false,
        context: '', contexts: [], active_context_id: null,
      };
      if (cmd === 'has_api_key') return true;
      if (cmd === 'list_sessions') return sampleSessions;
      if (cmd === 'list_mics' || cmd === 'list_loopback_sources') return [];
      return null;
    });

  const searchInput = async (container: HTMLElement) => {
    await waitFor(() => {
      expect(container.querySelector('input[type="search"]')).not.toBeNull();
    });
    return container.querySelector('input[type="search"]') as HTMLInputElement;
  };

  it('whitespace-only query keeps the full session list and issues no search IPC', async () => {
    invokeMock.mockClear();
    bootMock();
    const { container } = render(MainApp);
    await waitFor(() => {
      expect(container.querySelectorAll('button[data-active]').length).toBe(2);
    });
    const input = await searchInput(container);
    await fireEvent.input(input, { target: { value: '   ' } });
    // An untrimmed `query ? hits : sessions` gate blanked the sidebar here.
    expect(container.querySelectorAll('button[data-active]').length).toBe(2);
    await new Promise((r) => setTimeout(r, 250));
    expect(invokeMock).not.toHaveBeenCalledWith('search_transcripts', expect.anything());
  });

  it('debounces rapid keystrokes into a single search IPC call', async () => {
    invokeMock.mockClear();
    bootMock((cmd) => (cmd === 'search_transcripts' ? [sampleSessions[1]] : undefined));
    const { container } = render(MainApp);
    const input = await searchInput(container);
    await fireEvent.input(input, { target: { value: 'al' } });
    await fireEvent.input(input, { target: { value: 'alpha' } });
    await new Promise((r) => setTimeout(r, 320));
    const calls = invokeMock.mock.calls.filter((c) => c[0] === 'search_transcripts');
    expect(calls.length).toBe(1);
    expect(calls[0]![1]).toMatchObject({ query: 'alpha' });
  });

  it('a stale search response cannot overwrite a newer one', async () => {
    invokeMock.mockClear();
    const resolvers = new Map<string, (rows: unknown) => void>();
    bootMock((cmd, args) =>
      cmd === 'search_transcripts'
        ? new Promise((res) => resolvers.set(args.query, res))
        : undefined,
    );
    const { container } = render(MainApp);
    const input = await searchInput(container);

    await fireEvent.input(input, { target: { value: 'old' } });
    await new Promise((r) => setTimeout(r, 230)); // debounce fires; call 1 pending
    await fireEvent.input(input, { target: { value: 'new' } });
    await new Promise((r) => setTimeout(r, 230)); // call 2 pending
    expect(resolvers.has('old') && resolvers.has('new')).toBe(true);

    // Newer response lands first (ONE row), then the stale one (TWO rows)
    // arrives late — it must be discarded, leaving one row rendered.
    resolvers.get('new')!([sampleSessions[0]]);
    await new Promise((r) => setTimeout(r, 10));
    resolvers.get('old')!(sampleSessions);
    await new Promise((r) => setTimeout(r, 10));
    expect(container.querySelectorAll('button[data-active]').length).toBe(1);
  });
});

describe('MainApp past-session labels', () => {
  it('past viewer renders the stored session languages and mode, not the current config', async () => {
    const pastSession = {
      id: 42, started_at: Date.now() - 120_000, ended_at: Date.now() - 60_000,
      mode: 'meeting', lang_a: 'zh', lang_b: 'en', device_label: null,
      duration_ms: 60_000,
    };
    const token = {
      id: 1, session_id: 42, ts_ms: 1, text: 'hello', language: 'en',
      status: 'original', speaker: null, is_break: 0,
    };

    invokeMock.mockClear();
    (invokeMock as any).mockImplementation(async (cmd: string) => {
      if (cmd === 'get_config') return {
        // Current config deliberately DIFFERS from the stored session:
        language_a: 'en', language_b: 'vi',
        hotkey: 'Ctrl+Shift+V', theme: 'system',
        default_meeting_source: null, default_mic: null, meeting_capture_mic: false,
        mode: 'conversation', font_size: 'm', show_pinyin: false,
        context: '', contexts: [], active_context_id: null,
      };
      if (cmd === 'has_api_key') return true;
      if (cmd === 'list_sessions') return [pastSession];
      if (cmd === 'list_mics' || cmd === 'list_loopback_sources') return [];
      if (cmd === 'get_session') return { session: pastSession, tokens: [token] };
      return null;
    });

    const { transcript } = await import('../src/lib/stores.svelte');
    transcript.reset();

    const { container, findByText, queryByText } = render(MainApp);
    await waitFor(() => {
      expect(container.querySelectorAll('button[data-active]').length).toBeGreaterThan(0);
    });
    await fireEvent.click(container.querySelector('button[data-active]') as HTMLElement);

    // The stored row says zh→en MEETING; the meeting-mode pane header is
    // "ZH · multi-speaker". With the bug, the current en→vi conversation
    // config rendered the "EN/VI" conversation header instead.
    await findByText('ZH · multi-speaker');
    expect(queryByText('EN/VI')).toBeNull();
  });
});

describe('MainApp session lifecycle hygiene', () => {
  afterEach(async () => {
    const { session, transcript } = await import('../src/lib/stores.svelte');
    session.stop();
    transcript.reset();
  });

  const standardMock = () =>
    (invokeMock as any).mockImplementation(async (cmd: string) => {
      if (cmd === 'get_config') return {
        language_a: 'en', language_b: 'vi',
        hotkey: 'Ctrl+Shift+V', theme: 'system',
        default_meeting_source: null, default_mic: null, meeting_capture_mic: false,
        mode: 'meeting', font_size: 'm', show_pinyin: false,
        context: '', contexts: [], active_context_id: null,
      };
      if (cmd === 'has_api_key') return true;
      if (cmd === 'list_sessions') return sampleSessions;
      if (cmd === 'list_mics' || cmd === 'list_loopback_sources') return [];
      return null;
    });

  it('session-stopped clears the live partial and ignores stale session ids', async () => {
    invokeMock.mockClear();
    standardMock();
    const { findByText, queryByText } = render(MainApp);
    const { session } = await import('../src/lib/stores.svelte');

    emitEvent('voxtide://event', { kind: 'session-started', session_id: 7, mode: 'meeting' });
    emitEvent('voxtide://event', {
      kind: 'transcript-live', status: 'original', text: 'ghost partial',
      language: 'en', chip: null,
    });
    await findByText('ghost partial');

    // A stale SessionStopped from a detached OLD worker must be ignored.
    emitEvent('voxtide://event', { kind: 'session-stopped', session_id: 3, duration_ms: 1 });
    expect(session.recording).toBe(true);
    expect(queryByText('ghost partial')).not.toBeNull();

    // The real stop clears the blinking partial along with recording state.
    emitEvent('voxtide://event', { kind: 'session-stopped', session_id: 7, duration_ms: 100 });
    await waitFor(() => {
      expect(session.recording).toBe(false);
      expect(queryByText('ghost partial')).toBeNull();
    });
  });

  it('session-started refetches the sidebar list (live row appears)', async () => {
    invokeMock.mockClear();
    standardMock();
    render(MainApp);
    await waitFor(() => {
      expect(invokeMock.mock.calls.filter((c) => c[0] === 'list_sessions').length).toBe(1);
    });
    emitEvent('voxtide://event', { kind: 'session-started', session_id: 7, mode: 'meeting' });
    await waitFor(() => {
      expect(invokeMock.mock.calls.filter((c) => c[0] === 'list_sessions').length).toBe(2);
    });
  });

  it('overlay visibility events keep the toggle in sync', async () => {
    invokeMock.mockClear();
    standardMock();
    const { container } = render(MainApp);
    await waitFor(() => {
      expect(container.querySelector('button[title="Show overlay"]')).not.toBeNull();
    });
    // Backend reports the overlay as visible (e.g. it was shown elsewhere or
    // hid itself) — the toggle must track it, so the next click HIDES.
    emitEvent('voxtide://overlay', { visible: true });
    await fireEvent.click(container.querySelector('button[title="Show overlay"]') as HTMLElement);
    await waitFor(() => {
      expect(invokeMock.mock.calls.some((c) => c[0] === 'hide_overlay')).toBe(true);
    });
    expect(invokeMock.mock.calls.some((c) => c[0] === 'show_overlay')).toBe(false);
  });
});

describe('MainApp boot resilience', () => {
  afterEach(async () => {
    const { session, transcript } = await import('../src/lib/stores.svelte');
    session.stop();
    transcript.reset();
  });

  it('a failing config fetch still renders the app with an error strip and a live event listener', async () => {
    invokeMock.mockClear();
    invokeMock.mockImplementation(async (cmd: string) => {
      if (cmd === 'get_config') throw new Error('disk exploded');
      if (cmd === 'has_api_key') return true;
      if (cmd === 'list_sessions') return sampleSessions;
      if (cmd === 'list_mics' || cmd === 'list_loopback_sources') return [];
      return null;
    });
    const { findByTestId, findByText } = render(MainApp);
    // The boot failure surfaces in the T5 error strip instead of an
    // unhandled rejection that leaves the app inert.
    const strip = await findByTestId('app-error');
    expect(strip.textContent).toContain('startup:');
    expect(strip.textContent).toContain('disk exploded');
    // The core-event listener attached BEFORE the failing fetch — a backend
    // event still drives the UI:
    emitEvent('voxtide://event', { kind: 'session-started', session_id: 7, mode: 'meeting' });
    await findByText('Stop');
  });
});

describe('MainApp status bar truth', () => {
  it('renders the model reported by app_info, not a hardcoded literal', async () => {
    invokeMock.mockClear();
    invokeMock.mockImplementation(async (cmd: string) => {
      if (cmd === 'get_config') return {
        language_a: 'en', language_b: 'vi',
        hotkey: 'Ctrl+Shift+V', theme: 'system',
        default_meeting_source: null, default_mic: null, meeting_capture_mic: false,
        mode: 'meeting', font_size: 'm', show_pinyin: false,
        context: '', contexts: [], active_context_id: null,
      };
      if (cmd === 'has_api_key') return true;
      if (cmd === 'list_sessions') return sampleSessions;
      if (cmd === 'list_mics' || cmd === 'list_loopback_sources') return [];
      if (cmd === 'app_info') return { model: 'stt-rt-v9-future', sample_rate_hz: 16000, channels: 1 };
      return null;
    });
    const { container, findByText } = render(MainApp);
    // The status bar shows whatever the backend reports (single source of
    // truth: voxtide_core::translation::soniox::MODEL), so a model bump can
    // never leave the UI lying.
    await findByText(/stt-rt-v9-future/);
    expect(container.textContent ?? '').not.toContain('stt-rt-v5');
  });
});

describe('MainApp context presets', () => {
  // `devices` (mics/loopbacks) is a module singleton, same as `session` and
  // `transcript` below — it isn't reset by production code (in the real app
  // it only ever populates once, from empty, before `selectedSource` can be
  // set). Across tests in THIS file it persists, so without clearing it a
  // fresh MainApp instance's boot `$effect` sees the PRIOR test's leftover
  // device list as already non-empty and picks `selectedSource` immediately
  // — before that instance's own get_config/list_loopback_sources calls
  // resolve — letting a click fire onStart against a stale config. Both
  // start-path tests below use the same "System Audio" device label, which
  // is exactly what makes that race land. beforeEach protects test 1 from
  // whatever ran earlier in the file; afterEach protects tests after this
  // block (and keeps this block's own tests from depending on run order).
  const resetDevices = async () => {
    const { devices } = await import('../src/lib/stores.svelte');
    devices.setLoopbacks([]);
    devices.setMics([]);
  };
  beforeEach(resetDevices);
  afterEach(async () => {
    const { session, transcript } = await import('../src/lib/stores.svelte');
    session.stop();
    transcript.reset();
    await resetDevices();
  });

  const loopback = [{ id: 'sys', label: 'System Audio', default: true }];

  it('start sends the active preset\'s text as context, not the legacy blob', async () => {
    invokeMock.mockClear();
    invokeMock.mockImplementation(async (cmd: string) => {
      if (cmd === 'get_config') return {
        language_a: 'en', language_b: 'vi',
        hotkey: 'Ctrl+Shift+V', theme: 'system',
        default_meeting_source: null, default_mic: null, meeting_capture_mic: false,
        mode: 'meeting', font_size: 'm', show_pinyin: false,
        // Deliberately non-empty and distinct from the preset text: if a
        // regression sends `config.context` instead of the resolved preset,
        // this exact string would show up in the assertion below.
        context: 'stale legacy blob — must NOT be sent',
        contexts: [{ id: 'p1', name: 'Standup', text: 'Speakers: Nam, Yuki.' }],
        active_context_id: 'p1',
      };
      if (cmd === 'has_api_key') return true;
      if (cmd === 'list_sessions') return [];
      if (cmd === 'list_mics') return [];
      if (cmd === 'list_loopback_sources') return loopback;
      if (cmd === 'start_session') return 1;
      return null;
    });

    const { container } = render(MainApp);
    const scoped = within(container);
    await waitForStartReady(container, 'System Audio');
    await fireEvent.click(scoped.getByRole('button', { name: /Start/ }));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith('start_session', expect.objectContaining({
        req: expect.objectContaining({ context: 'Speakers: Nam, Yuki.' }),
      }));
    });
  });

  it('start sends an empty context when active_context_id is null', async () => {
    invokeMock.mockClear();
    invokeMock.mockImplementation(async (cmd: string) => {
      if (cmd === 'get_config') return {
        language_a: 'en', language_b: 'vi',
        hotkey: 'Ctrl+Shift+V', theme: 'system',
        default_meeting_source: null, default_mic: null, meeting_capture_mic: false,
        mode: 'meeting', font_size: 'm', show_pinyin: false,
        context: '',
        contexts: [{ id: 'p1', name: 'Standup', text: 'Speakers: Nam, Yuki.' }],
        active_context_id: null,
      };
      if (cmd === 'has_api_key') return true;
      if (cmd === 'list_sessions') return [];
      if (cmd === 'list_mics') return [];
      if (cmd === 'list_loopback_sources') return loopback;
      if (cmd === 'start_session') return 1;
      return null;
    });

    const { container } = render(MainApp);
    const scoped = within(container);
    await waitForStartReady(container, 'System Audio');
    await fireEvent.click(scoped.getByRole('button', { name: /Start/ }));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith('start_session', expect.objectContaining({
        req: expect.objectContaining({ context: '' }),
      }));
    });
  });

  it('boot with a legacy context blob seeds exactly one preset and persists it once', async () => {
    invokeMock.mockClear();
    invokeMock.mockImplementation(async (cmd: string) => {
      if (cmd === 'get_config') return {
        language_a: 'en', language_b: 'vi',
        hotkey: 'Ctrl+Shift+V', theme: 'system',
        default_meeting_source: null, default_mic: null, meeting_capture_mic: false,
        mode: 'meeting', font_size: 'm', show_pinyin: false,
        context: 'Old global context',
        contexts: [], active_context_id: null,
      };
      if (cmd === 'has_api_key') return true;
      if (cmd === 'list_sessions') return [];
      if (cmd === 'list_mics' || cmd === 'list_loopback_sources') return [];
      return null;
    });

    render(MainApp);

    // The seed persist is fire-and-forget (`void persist(...)`) — wait for
    // the resulting set_config IPC call rather than asserting synchronously.
    await waitFor(() => {
      expect(invokeMock.mock.calls.some((c) => c[0] === 'set_config')).toBe(true);
    });

    const setConfigCalls = invokeMock.mock.calls.filter((c) => c[0] === 'set_config');
    expect(setConfigCalls).toHaveLength(1);
    const cfgArg = (setConfigCalls[0]![1] as any).cfg;
    expect(cfgArg.contexts).toHaveLength(1);
    expect(cfgArg.contexts[0]).toMatchObject({ name: 'My context', text: 'Old global context' });
    // Don't hard-code the generated uuid — assert the cross-reference instead.
    expect(cfgArg.active_context_id).toBe(cfgArg.contexts[0]!.id);
    expect(cfgArg.context).toBe('');
  });
});

describe('MainApp mid-session context switch', () => {
  // `session` is a module singleton (see the "MainApp context presets" block
  // above) — stop it after every test so a "recording" test can't bleed into
  // a later one, in either direction.
  afterEach(async () => {
    const { session, transcript } = await import('../src/lib/stores.svelte');
    session.stop();
    transcript.reset();
  });

  const contextsConfig = {
    language_a: 'en', language_b: 'vi',
    hotkey: 'Ctrl+Shift+V', theme: 'system',
    default_meeting_source: null, default_mic: null, meeting_capture_mic: false,
    mode: 'meeting', font_size: 'm', show_pinyin: false,
    context: '',
    contexts: [{ id: 'p1', name: 'Standup', text: 'Speakers: Nam, Yuki.' }],
    // Widen from the inferred `null` literal: bootMock's override below
    // (`{ ...contextsConfig, active_context_id: 'p1' }`) needs the field's
    // static type to be `string | null`, matching the real AppConfig shape.
    active_context_id: null as string | null,
  };
  // Optional override so a test can boot with a non-null `active_context_id`
  // (e.g. to exercise "re-pick the already-active preset") without duplicating
  // the whole mockImplementation.
  const bootMock = (cfg: typeof contextsConfig = contextsConfig) =>
    (invokeMock as any).mockImplementation(async (cmd: string) => {
      if (cmd === 'get_config') return cfg;
      if (cmd === 'has_api_key') return true;
      if (cmd === 'list_sessions') return [];
      if (cmd === 'list_mics' || cmd === 'list_loopback_sources') return [];
      return null;
    });

  // The trigger's "No context" label renders immediately (before get_config
  // even resolves — activeId defaults to null either way), so it's only a
  // mount gate. "Standup" only exists once the boot fetch lands and populates
  // `contexts`, so `findByText` (which polls) is the real config-loaded gate —
  // a plain `getByText` here would race the boot promise chain and could open
  // the panel while `contexts` is still `[]`.
  async function pickStandup(findByText: (text: string) => Promise<HTMLElement>) {
    const trigger = await findByText('No context');
    await fireEvent.click(trigger);
    const option = await findByText('Standup');
    await fireEvent.click(option);
  }

  it('picking a context WHILE RECORDING calls update_context with the picked preset\'s text', async () => {
    invokeMock.mockClear();
    bootMock();
    const { findByText } = render(MainApp);
    const { session } = await import('../src/lib/stores.svelte');
    session.start(7, Date.now());

    await pickStandup(findByText);

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith('update_context', { text: 'Speakers: Nam, Yuki.' });
    });
    // The pick still persists as the default for next time, same as stopped.
    expect(invokeMock).toHaveBeenCalledWith('set_config', expect.objectContaining({
      cfg: expect.objectContaining({ active_context_id: 'p1' }),
    }));
  });

  it('picking a context WHILE STOPPED only persists — no update_context call', async () => {
    invokeMock.mockClear();
    bootMock();
    const { findByText } = render(MainApp);
    const { session } = await import('../src/lib/stores.svelte');
    session.stop(); // belt-and-braces: this store is a module singleton shared across tests.

    await pickStandup(findByText);

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith('set_config', expect.objectContaining({
        cfg: expect.objectContaining({ active_context_id: 'p1' }),
      }));
    });
    expect(invokeMock).not.toHaveBeenCalledWith('update_context', expect.anything());
  });

  // Once `active_context_id` is already 'p1', the trigger's own label reads
  // "Standup" too (see ContextPicker's `triggerLabel`) — the SAME text as the
  // dropdown option once the panel opens. A second `findByText('Standup')`
  // would then see two matches and throw "multiple elements found", so grab
  // the trigger node up front and, once the panel is open, click whichever
  // "Standup" match ISN'T that same node — robust regardless of DOM order.
  async function reopenAndRepickStandup(
    container: HTMLElement,
    findByText: (text: string) => Promise<HTMLElement>,
  ) {
    const trigger = await findByText('Standup');
    await fireEvent.click(trigger);
    const matches = within(container).getAllByText('Standup');
    const option = matches.find((el) => el !== trigger);
    if (!option) throw new Error('dropdown "Standup" option not found');
    await fireEvent.click(option);
  }

  it('re-picking the ALREADY-active context WHILE RECORDING does not call update_context', async () => {
    invokeMock.mockClear();
    bootMock({ ...contextsConfig, active_context_id: 'p1' });
    const { container, findByText } = render(MainApp);
    const { session } = await import('../src/lib/stores.svelte');
    session.start(7, Date.now());

    await reopenAndRepickStandup(container, findByText);

    // persist() is unconditional and idempotent — re-picking the same id still
    // saves (same shape as every other pick).
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith('set_config', expect.objectContaining({
        cfg: expect.objectContaining({ active_context_id: 'p1' }),
      }));
    });
    // But the selection didn't actually change, so the mid-session reconnect
    // must be suppressed — no update_context call, no dropped audio tail.
    expect(invokeMock).not.toHaveBeenCalledWith('update_context', expect.anything());
  });
});
