import { describe, it, expect, vi } from 'vitest';
import { render, waitFor } from '@testing-library/svelte';
import OverlayWindow from '../src/components/overlay/OverlayWindow.svelte';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(async (cmd: string): Promise<unknown> => {
    if (cmd === 'get_config') return {
      language_a: 'ja', language_b: 'ko',
      hotkey: 'CommandOrControl+Shift+V', theme: 'light',
      default_meeting_source: null, default_mic: null,
      mode: 'meeting', font_size: 'm', show_pinyin: false,
    };
    return null;
  }),
}));
vi.mock('@tauri-apps/api/core', () => ({ invoke: invokeMock }));

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

import OverlayApp from '../src/routes/OverlayApp.svelte';

describe('OverlayWindow', () => {
  const base = {
    lines: [
      'Được rồi, hãy bắt đầu.',
      'Chúng tôi đang đi trước tiến độ.',
      'Độ trễ tổng từ đầu đến cuối hiện đang khoảng hai trăm sáu mươi mili-giây.',
      'Con số đó thấp hơn nhiều so với mục tiêu dưới một giây.',
      'Cột mốc tiếp theo là phát hành bản nguyên mẫu vào cuối tuần.',
    ],
    state: 'active' as const,
    connectionLabel: 'EN → VI',
    onclose: () => {},
  };

  it('renders all 5 lines when active', () => {
    const { container } = render(OverlayWindow, { props: { ...base, hover: false } });
    const text = container.textContent ?? '';
    for (const l of base.lines) expect(text).toContain(l);
  });

  it('reveals the hover strip when hover is true', () => {
    const { container } = render(OverlayWindow, { props: { ...base, hover: true } });
    expect(container.querySelector('[data-strip="visible"]')).toBeTruthy();
  });

  it('shows language-neutral reconnecting copy when state=reconnecting', () => {
    const { getByText } = render(OverlayWindow, {
      props: { ...base, state: 'reconnecting', hover: false, attempt: 2, retryInMs: 1000 },
    });
    expect(getByText(/Reconnecting…/)).toBeTruthy();
  });
});

describe('OverlayApp', () => {
  const setTauri = () => {
    (window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ = {};
  };
  const clearTauri = () => {
    delete (window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
  };

  it('segments on utterance breaks, never on punctuation', async () => {
    setTauri();
    try {
      const { container } = render(OverlayApp);
      await waitFor(() => expect(listeners.has('voxtide://event')).toBe(true));
      emitEvent('voxtide://event', { kind: 'session-started', session_id: 1, mode: 'meeting' });
      emitEvent('voxtide://event', {
        kind: 'transcript-final', status: 'translation', text: 'Câu một.',
        language: 'vi', chip: 'A', ts_ms: 1,
      });
      emitEvent('voxtide://event', {
        kind: 'transcript-final', status: 'translation', text: ' vẫn câu một',
        language: 'vi', chip: 'A', ts_ms: 2,
      });
      await waitFor(() => {
        // The trailing '.' must NOT have flushed: still one growing line.
        expect(container.textContent).toContain('Câu một. vẫn câu một');
      });
      emitEvent('voxtide://event', { kind: 'utterance-break' });
      emitEvent('voxtide://event', {
        kind: 'transcript-final', status: 'translation', text: 'Câu hai',
        language: 'vi', chip: 'A', ts_ms: 3,
      });
      await waitFor(() => {
        const text = container.textContent ?? '';
        expect(text).toContain('Câu một. vẫn câu một');
        expect(text).toContain('Câu hai');
        // The break put them on SEPARATE lines (not concatenated).
        expect(text).not.toContain('Câu một. vẫn câu mộtCâu hai');
      });
    } finally {
      clearTauri();
    }
  });

  it('derives pair label, hotkey hint and theme from config; follows config events live', async () => {
    setTauri();
    try {
      const { container } = render(OverlayApp);
      // From get_config (ja→ko meeting, light theme, default hotkey). The
      // idle hint renders the hotkey (jsdom platform is non-mac → names):
      await waitFor(() => {
        expect(container.textContent).toContain('Ctrl+Shift+V');
        expect(document.body.classList.contains('vt-theme-light')).toBe(true);
      });
      // The pair label shows in the strip while a session is active:
      await waitFor(() => expect(listeners.has('voxtide://event')).toBe(true));
      emitEvent('voxtide://event', { kind: 'session-started', session_id: 1, mode: 'meeting' });
      await waitFor(() => {
        expect(container.textContent).toContain('JA → KO');
      });

      // A config change re-derives everything without a restart:
      emitEvent('voxtide://config', {
        language_a: 'zh', language_b: 'en', hotkey: 'Alt+F5', theme: 'dark',
        default_meeting_source: null, default_mic: null, mode: 'conversation',
        font_size: 'm', show_pinyin: false,
      });
      await waitFor(() => {
        expect(container.textContent).toContain('ZH ⇄ EN');
        expect(document.body.classList.contains('vt-theme-dark')).toBe(true);
      });
      // The hotkey hint lives in the idle view — stop the session to see it:
      emitEvent('voxtide://event', { kind: 'session-stopped', session_id: 1, duration_ms: 5 });
      await waitFor(() => {
        expect(container.textContent).toContain('Alt+F5');
      });
    } finally {
      clearTauri();
    }
  });
});
