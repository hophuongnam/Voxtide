import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(async (cmd: string, args: any) => {
    if (cmd === 'set_api_key') return null;
    if (cmd === 'has_api_key') return true;
    if (cmd === 'get_config') return {
      language_a: 'en', language_b: 'vi', mine: 'b',
      hotkey: 'Ctrl+Shift+V', theme: 'system',
      default_meeting_source: null, default_mic: null,
    };
    if (cmd === 'set_config') return null;
    return null;
  }),
}));
vi.mock('@tauri-apps/api/core', () => ({ invoke: invokeMock }));

import SettingsSheet from '../src/components/settings/SettingsSheet.svelte';

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

  it('switches theme', async () => {
    const { findByText } = render(SettingsSheet, { props: { open: true, onclose: () => {} } });
    await fireEvent.click(await findByText('dark'));
    // theme change persists via set_config
    expect(invokeMock).toHaveBeenCalledWith('set_config', expect.objectContaining({
      cfg: expect.objectContaining({ theme: 'dark' }),
    }));
  });
});
