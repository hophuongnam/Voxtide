import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import TranscriptPane from '../src/components/transcript/TranscriptPane.svelte';
import EmptyState from '../src/components/transcript/EmptyState.svelte';
import NoApiKey from '../src/components/transcript/NoApiKey.svelte';
import type { AppConfig } from '../src/types';

const sampleCfg: AppConfig = {
  language_a: 'zh', language_b: 'en', mine: 'a', hotkey: 'Ctrl+Shift+V',
  theme: 'system', default_meeting_source: null, default_mic: null,
  mode: 'conversation', font_size: 'm', show_pinyin: false,
};

describe('TranscriptPane', () => {
  it('renders both columns with header labels', () => {
    const { getByText } = render(TranscriptPane, {
      props: {
        mode: 'conversation',
        a: { code: 'EN', name: 'English' },
        b: { code: 'JA', name: 'Japanese' },
        mine: 'a',
        original: [{ ts_ms: 100, status: 'original', text: 'Hello', language: 'en', chip: 'A', live: false }],
        translation: [{ ts_ms: 100, status: 'translation', text: 'こんにちは', language: 'ja', chip: 'A', live: false }],
        liveOriginal: '', liveTranslation: '',
      },
    });
    expect(getByText('Original')).toBeTruthy();
    expect(getByText('Translation')).toBeTruthy();
    expect(getByText('Hello')).toBeTruthy();
    expect(getByText('こんにちは')).toBeTruthy();
  });

  it('sets the transcript font-size CSS variable from the fontSize prop', () => {
    const { container } = render(TranscriptPane, {
      props: {
        mode: 'conversation',
        a: { code: 'ZH', name: 'Chinese' }, b: { code: 'EN', name: 'English' },
        mine: 'a', original: [], translation: [], liveOriginal: '', liveTranslation: '',
        fontSize: 'xl', showPinyin: false, cfg: null, onconfigchange: () => {},
      },
    });
    const root = container.querySelector('[data-testid="transcript-root"]') as HTMLElement;
    expect(root.style.getPropertyValue('--vt-transcript-size')).toBe('19px');
  });

  it('renders pinyin ruby in zh lines when showPinyin is on', () => {
    const { container } = render(TranscriptPane, {
      props: {
        mode: 'conversation',
        a: { code: 'ZH', name: 'Chinese' }, b: { code: 'EN', name: 'English' },
        mine: 'a',
        original: [{ ts_ms: 0, status: 'original', text: '你好', language: 'zh', chip: null, live: false }],
        translation: [], liveOriginal: '', liveTranslation: '',
        fontSize: 'm', showPinyin: true, cfg: null, onconfigchange: () => {},
      },
    });
    expect(container.querySelectorAll('ruby').length).toBe(2);
  });

  it('Aa popover toggles and emits config changes', async () => {
    const onconfigchange = vi.fn();
    const { getAllByText, getByText } = render(TranscriptPane, {
      props: {
        mode: 'conversation',
        a: { code: 'ZH', name: 'Chinese' }, b: { code: 'EN', name: 'English' },
        mine: 'a', original: [], translation: [], liveOriginal: '', liveTranslation: '',
        fontSize: 'm', showPinyin: false, cfg: sampleCfg, onconfigchange,
      },
    });
    const aa = getAllByText('Aa');           // one per column
    expect(aa.length).toBe(2);
    await fireEvent.click(aa[0]!);
    await fireEvent.click(getByText(/Show pinyin/));
    expect(onconfigchange).toHaveBeenCalledWith(
      expect.objectContaining({ show_pinyin: true }),
    );
  });
});

describe('EmptyState', () => {
  it('shows mode-specific copy', () => {
    const { getByText } = render(EmptyState, { props: { mode: 'meeting' } });
    expect(getByText(/Ready to translate a meeting/)).toBeTruthy();
  });
});

describe('NoApiKey', () => {
  it('lists the Add API key CTA', () => {
    const { getByText } = render(NoApiKey, { props: { onaddkey: () => {} } });
    expect(getByText('Add API key')).toBeTruthy();
  });
});
