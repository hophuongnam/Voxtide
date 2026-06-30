import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, waitFor } from '@testing-library/svelte';

const { invokeMock, listenMock, startMicCaptureMock, stopMicCaptureMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(async (cmd: string): Promise<unknown> => {
    if (cmd === 'get_config') return {
      language_a: 'en',
      language_b: 'vi',
      hotkey: 'Ctrl+Shift+V',
      theme: 'system',
      default_meeting_source: null,
      default_mic: null,
      mode: 'conversation',
      font_size: 'm',
      show_pinyin: false,
      meeting_capture_mic: false,
      mic_gain: 1,
      mic_agc: false,
    };
    if (cmd === 'has_api_key') return true;
    if (cmd === 'list_sessions') return [];
    return null;
  }),
  listenMock: vi.fn(async () => () => {}),
  startMicCaptureMock: vi.fn(async (onStats?: (s: unknown) => void) => {
    onStats?.({ state: 'running', sampleRate: 16000, batches: 1 });
  }),
  stopMicCaptureMock: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({ invoke: invokeMock }));
vi.mock('@tauri-apps/api/event', () => ({ listen: listenMock }));
vi.mock('@tauri-apps/api/app', () => ({ getVersion: async () => '0.1.8' }));
vi.mock('@tauri-apps/plugin-opener', () => ({ openUrl: vi.fn() }));
vi.mock('../src/lib/miccapture', () => ({
  startMicCapture: startMicCaptureMock,
  stopMicCapture: stopMicCaptureMock,
  setMicGain: vi.fn(),
  setMicAgc: vi.fn(),
}));

import FaceToFaceView from '../src/routes/FaceToFaceView.svelte';

describe('FaceToFaceView Android recording flow', () => {
  afterEach(() => {
    cleanup();
    invokeMock.mockClear();
    listenMock.mockClear();
    startMicCaptureMock.mockClear();
    stopMicCaptureMock.mockClear();
    delete (window as any).__TAURI_INTERNALS__;
  });

  it('acquires WebView mic capture before opening the Rust session', async () => {
    (window as any).__TAURI_INTERNALS__ = {};
    const order: string[] = [];
    startMicCaptureMock.mockImplementationOnce(async (onStats?: (s: unknown) => void) => {
      order.push('mic');
      onStats?.({ state: 'running', sampleRate: 16000, batches: 1 });
    });
    invokeMock.mockImplementation(async (cmd: string) => {
      if (cmd === 'start_session') order.push('session');
      return cmd === 'get_config' ? {
        language_a: 'en', language_b: 'vi', hotkey: 'Ctrl+Shift+V', theme: 'system',
        default_meeting_source: null, default_mic: null, mode: 'conversation',
        font_size: 'm', show_pinyin: false, meeting_capture_mic: false, mic_gain: 1, mic_agc: false,
      } : cmd === 'has_api_key' ? true : null;
    });

    const { getByRole } = render(FaceToFaceView);
    await waitFor(() => expect(getByRole('button', { name: 'Record' })).toBeInTheDocument());
    await fireEvent.click(getByRole('button', { name: 'Record' }));

    await waitFor(() => expect(order).toEqual(['mic', 'session']));
  });

  it('stops mic capture and the backend session on Android lifecycle stop', async () => {
    (window as any).__TAURI_INTERNALS__ = {};
    const { getByRole } = render(FaceToFaceView);
    await waitFor(() => expect(getByRole('button', { name: 'Record' })).toBeInTheDocument());
    await fireEvent.click(getByRole('button', { name: 'Record' }));
    await waitFor(() => expect(startMicCaptureMock).toHaveBeenCalled());

    window.dispatchEvent(new Event('voxtide:android-stop'));

    await waitFor(() => {
      expect(stopMicCaptureMock).toHaveBeenCalled();
      expect(invokeMock).toHaveBeenCalledWith('stop_session');
    });
  });
});
