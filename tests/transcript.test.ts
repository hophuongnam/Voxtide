import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import TranscriptPane from '../src/components/transcript/TranscriptPane.svelte';
import EmptyState from '../src/components/transcript/EmptyState.svelte';
import NoApiKey from '../src/components/transcript/NoApiKey.svelte';
import type { AppConfig } from '../src/types';

const sampleCfg: AppConfig = {
  language_a: 'zh', language_b: 'en', hotkey: 'Ctrl+Shift+V',
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

  it('meeting mode labels Original as source (a) and Translation as target (b)', () => {
    // Regression: a meeting where Vietnamese is spoken and English is the
    // translation target must show Original=VI, Translation=EN (never VI/VI).
    const { getByText } = render(TranscriptPane, {
      props: {
        mode: 'meeting',
        a: { code: 'VI', name: 'Vietnamese' },
        b: { code: 'EN', name: 'English' },
        original: [], translation: [], liveOriginal: '', liveTranslation: '',
      },
    });
    expect(getByText('VI · multi-speaker')).toBeTruthy();
    expect(getByText('EN')).toBeTruthy();
  });

  it('sets the transcript font-size CSS variable from the fontSize prop', () => {
    const { container } = render(TranscriptPane, {
      props: {
        mode: 'conversation',
        a: { code: 'ZH', name: 'Chinese' }, b: { code: 'EN', name: 'English' },
        original: [], translation: [], liveOriginal: '', liveTranslation: '',
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
        original: [], translation: [], liveOriginal: '', liveTranslation: '',
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
    expect(getByText(/Ready to translate system audio/)).toBeTruthy();
  });
});

describe('NoApiKey', () => {
  it('lists the Add API key CTA', () => {
    const { getByText } = render(NoApiKey, { props: { onaddkey: () => {} } });
    expect(getByText('Add API key')).toBeTruthy();
  });
});

describe('TranscriptPane live-line language', () => {
  it('uses the detected live language over the column default (pinyin on live zh under a non-zh column)', () => {
    const { container } = render(TranscriptPane, {
      props: {
        mode: 'conversation',
        // Column default says EN — but the DETECTED live language is zh.
        a: { code: 'EN', name: 'English' },
        b: { code: 'VI', name: 'Vietnamese' },
        original: [], translation: [],
        liveOriginal: '你好', liveTranslation: '',
        liveOriginalLang: 'zh',
        showPinyin: true,
        cfg: sampleCfg,
      },
    });
    expect(container.querySelectorAll('ruby').length).toBeGreaterThan(0);
  });
});

describe('TranscriptPane follow-tail', () => {
  const mkLines = (n: number, prefix: string, status: 'original' | 'translation') =>
    Array.from({ length: n }, (_, i) => ({
      ts_ms: i, status, text: `${prefix}${i}`, language: 'en', chip: null, live: false,
    }));
  const baseProps = (nOrig: number) => ({
    mode: 'meeting' as const,
    a: { code: 'EN', name: 'English' },
    b: { code: 'VI', name: 'Vietnamese' },
    original: mkLines(nOrig, 'o', 'original'),
    translation: mkLines(20, 't', 'translation'),
    liveOriginal: '', liveTranslation: '',
  });
  const raf = () => new Promise<void>((r) => requestAnimationFrame(() => r()));

  it('re-engages from the right column; mirror echoes never override the user column', async () => {
    const { container, rerender } = render(TranscriptPane, { props: baseProps(20) });
    const [left, right] = Array.from(container.querySelectorAll('.overflow-auto')) as HTMLElement[];
    // Fake geometry: TALL left (1000), SHORT right (500), 100px viewports.
    const geom = (el: HTMLElement, scrollHeight: number) => {
      Object.defineProperty(el, 'scrollHeight', { value: scrollHeight, configurable: true });
      Object.defineProperty(el, 'clientHeight', { value: 100, configurable: true });
    };
    geom(left!, 1000); geom(right!, 500);
    // Drain the mount-time auto-scroll (autoScrolling guard clears on 2nd rAF).
    await raf(); await raf(); await raf();

    // User scrolls the TALL column up, away from its bottom → disengage.
    left!.scrollTop = 500; // 500+100 < 1000-32
    left!.dispatchEvent(new Event('scroll'));
    // The mirror's echo on the SHORT column (clamped near ITS bottom) fires
    // while syncing — it must NOT re-engage follow-tail.
    right!.scrollTop = 460; // 460+100 >= 500-32 → "near bottom" if judged
    right!.dispatchEvent(new Event('scroll'));

    await rerender(baseProps(21)); // growth: would snap if (wrongly) engaged
    await raf(); await raf(); await raf();
    expect(left!.scrollTop).toBe(500); // no snap → still disengaged

    // Now the USER scrolls the right column to its bottom → re-engage.
    await raf(); // let the previous mirror's syncing flag clear
    right!.scrollTop = 470;
    right!.dispatchEvent(new Event('scroll'));
    await rerender(baseProps(22));
    await raf(); await raf(); await raf();
    expect(left!.scrollTop).toBe(1000); // snapped → re-engaged from the right
  });
});
