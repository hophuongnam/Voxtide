import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import ContextSection from '../src/components/settings/ContextSection.svelte';
import type { AppConfig } from '../src/types';

const base: AppConfig = {
  language_a: 'en', language_b: 'vi', hotkey: 'Ctrl+Shift+V',
  theme: 'system', default_meeting_source: null, default_mic: null, meeting_capture_mic: false,
  mode: 'conversation', font_size: 'm', show_pinyin: false, mic_gain: 1, mic_agc: false, context: '',
};

describe('ContextSection', () => {
  it('seeds the textarea from cfg.context', () => {
    const { container } = render(ContextSection, {
      props: { cfg: { ...base, context: 'Acme Corp' }, onchange: vi.fn() },
    });
    expect(container.querySelector('textarea')!.value).toBe('Acme Corp');
  });

  it('commits a trimmed change on blur', async () => {
    const onchange = vi.fn();
    const { container } = render(ContextSection, { props: { cfg: base, onchange } });
    const ta = container.querySelector('textarea')!;
    await fireEvent.input(ta, { target: { value: '  Speakers: Nam  ' } });
    await fireEvent.blur(ta);
    expect(onchange).toHaveBeenCalledWith(expect.objectContaining({ context: 'Speakers: Nam' }));
  });

  it('does NOT persist when the value is unchanged (no redundant disk write)', async () => {
    const onchange = vi.fn();
    const { container } = render(ContextSection, {
      props: { cfg: { ...base, context: 'unchanged' }, onchange },
    });
    const ta = container.querySelector('textarea')!;
    await fireEvent.blur(ta); // blur without editing
    expect(onchange).not.toHaveBeenCalled();
  });
});
