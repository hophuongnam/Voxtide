import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import ReadingControls from '../src/components/transcript/ReadingControls.svelte';
import type { AppConfig } from '../src/types';

const cfg: AppConfig = {
  language_a: 'en', language_b: 'vi', hotkey: 'Ctrl+Shift+V',
  theme: 'system', default_meeting_source: null, default_mic: null, meeting_capture_mic: false,
  mode: 'meeting', font_size: 'm', show_pinyin: false, mic_gain: 1, mic_agc: false, context: '',
  contexts: [], active_context_id: null,
};

describe('ReadingControls', () => {
  it('emits a font_size change when a size button is clicked', async () => {
    const onchange = vi.fn();
    const { getByText } = render(ReadingControls, { props: { cfg, onchange } });
    await fireEvent.click(getByText('xl'));
    expect(onchange).toHaveBeenCalledWith(
      expect.objectContaining({ font_size: 'xl' }),
    );
  });

  it('toggles show_pinyin', async () => {
    const onchange = vi.fn();
    const { getByText } = render(ReadingControls, { props: { cfg, onchange } });
    await fireEvent.click(getByText(/Show pinyin/));
    expect(onchange).toHaveBeenCalledWith(
      expect.objectContaining({ show_pinyin: true }),
    );
  });
});
