import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(async (cmd: string, args: any) => {
    if (cmd === 'set_api_key') return null;
    if (cmd === 'has_api_key') return true;
    if (cmd === 'get_config') return {
      language_a: 'en', language_b: 'vi',
      hotkey: 'Ctrl+Shift+V', theme: 'system',
      default_meeting_source: null, default_mic: null, meeting_capture_mic: false,
      mode: 'meeting', font_size: 'm', show_pinyin: false,
    };
    if (cmd === 'set_config') return null;
    return null;
  }),
}));
vi.mock('@tauri-apps/api/core', () => ({ invoke: invokeMock }));

import SettingsSheet from '../src/components/settings/SettingsSheet.svelte';
import { config } from '../src/lib/stores.svelte';

describe('SettingsSheet', () => {
  it('saves an API key when Save is pressed', async () => {
    const { findByLabelText, findByText } = render(SettingsSheet, {
      props: { open: true, onclose: () => {} },
    });
    const input = await findByLabelText('Soniox API key') as HTMLInputElement;
    await fireEvent.input(input, { target: { value: 'sk_live_test' } });
    await fireEvent.click(await findByText('Save'));
    expect(invokeMock).toHaveBeenCalledWith('set_api_key', { account: 'default', secret: 'sk_live_test' });
  });

  it('saving the API key updates the global config.hasApiKey store', async () => {
    config.setHasApiKey(false);
    const { findByLabelText, findByText } = render(SettingsSheet, {
      props: { open: true, onclose: () => {} },
    });
    const input = await findByLabelText('Soniox API key') as HTMLInputElement;
    await fireEvent.input(input, { target: { value: 'sk_live_test' } });
    await fireEvent.click(await findByText('Save'));
    // The reload() that fires after save must push hasApiKey=true to the global store
    // so MainApp's onStart guard (which reads config.hasApiKey) doesn't bail out.
    await new Promise(r => setTimeout(r, 10));
    expect(config.hasApiKey).toBe(true);
  });

  it('switches theme', async () => {
    const { findByText } = render(SettingsSheet, { props: { open: true, onclose: () => {} } });
    await fireEvent.click(await findByText('dark'));
    // theme change persists via set_config
    expect(invokeMock).toHaveBeenCalledWith('set_config', expect.objectContaining({
      cfg: expect.objectContaining({ theme: 'dark' }),
    }));
  });

  it('changing Source language select persists via set_config', async () => {
    const { findByLabelText } = render(SettingsSheet, { props: { open: true, onclose: () => {} } });
    const sel = await findByLabelText('Source language code') as HTMLSelectElement;
    await fireEvent.change(sel, { target: { value: 'ja' } });
    expect(invokeMock).toHaveBeenCalledWith('set_config', expect.objectContaining({
      cfg: expect.objectContaining({ language_a: 'ja' }),
    }));
  });

  it('changing Target language select persists via set_config', async () => {
    const { findByLabelText } = render(SettingsSheet, { props: { open: true, onclose: () => {} } });
    const sel = await findByLabelText('Target language code') as HTMLSelectElement;
    await fireEvent.change(sel, { target: { value: 'ja' } });
    expect(invokeMock).toHaveBeenCalledWith('set_config', expect.objectContaining({
      cfg: expect.objectContaining({ language_b: 'ja' }),
    }));
  });

  it('Reading section toggles pinyin and persists via set_config', async () => {
    const { findByText } = render(SettingsSheet, { props: { open: true, onclose: () => {} } });
    await fireEvent.click(await findByText(/Show pinyin/));
    expect(invokeMock).toHaveBeenCalledWith('set_config', expect.objectContaining({
      cfg: expect.objectContaining({ show_pinyin: true }),
    }));
  });

  it('Reading section changes font size and persists via set_config', async () => {
    const { findByText, getByText } = render(SettingsSheet, { props: { open: true, onclose: () => {} } });
    await findByText('Reading');
    await fireEvent.click(getByText('xl'));
    expect(invokeMock).toHaveBeenCalledWith('set_config', expect.objectContaining({
      cfg: expect.objectContaining({ font_size: 'xl' }),
    }));
  });

  it('hotkey field commits on change, not per keystroke', async () => {
    const { findByDisplayValue } = render(SettingsSheet, { props: { open: true, onclose: () => {} } });
    const input = await findByDisplayValue('Ctrl+Shift+V') as HTMLInputElement;
    invokeMock.mockClear();
    // Typing must NOT persist — an oninput save wrote garbage like
    // "Ctrl+Shif" to disk on every keystroke.
    await fireEvent.input(input, { target: { value: 'Alt+F' } });
    expect(invokeMock).not.toHaveBeenCalledWith('set_config', expect.anything());
    // Committing (Enter/blur → change) persists exactly once.
    await fireEvent.change(input, { target: { value: 'Alt+F5' } });
    expect(invokeMock).toHaveBeenCalledWith('set_config', expect.objectContaining({
      cfg: expect.objectContaining({ hotkey: 'Alt+F5' }),
    }));
  });

  it('hotkey rejection from the backend renders an inline error instead of throwing', async () => {
    const { findByDisplayValue, findByText } = render(SettingsSheet, { props: { open: true, onclose: () => {} } });
    const input = await findByDisplayValue('Ctrl+Shift+V') as HTMLInputElement;
    invokeMock.mockImplementationOnce(async (cmd: string) => {
      if (cmd === 'set_config') {
        throw { kind: 'invalid-hotkey', message: "could not register 'Bogus'" };
      }
      return null;
    });
    await fireEvent.change(input, { target: { value: 'Bogus' } });
    await findByText(/could not register/);
  });
});
