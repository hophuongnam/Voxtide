import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';
import TranscriptPane from '../src/components/transcript/TranscriptPane.svelte';
import EmptyState from '../src/components/transcript/EmptyState.svelte';
import NoApiKey from '../src/components/transcript/NoApiKey.svelte';
import type { AppConfig } from '../src/types';

const sampleCfg: AppConfig = {
  language_a: 'zh', language_b: 'en', hotkey: 'Ctrl+Shift+V',
  theme: 'system', default_meeting_source: null, default_mic: null, meeting_capture_mic: false,
  mode: 'conversation', font_size: 'm', show_pinyin: false, mic_gain: 1, mic_agc: false, context: '',
  contexts: [], active_context_id: null,
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
  it('uses the detected live language over the column default (pinyin on live zh under a non-zh column)', async () => {
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
    // Live conversion is debounced 150ms (perf): pinyin lands shortly after.
    await waitFor(() => {
      expect(container.querySelectorAll('ruby').length).toBeGreaterThan(0);
    });
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

  const geom = (el: HTMLElement, scrollHeight: number) => {
    Object.defineProperty(el, 'scrollHeight', { value: scrollHeight, configurable: true });
    Object.defineProperty(el, 'clientHeight', { value: 100, configurable: true });
  };

  it('stays engaged when a growth-induced scroll echo fires (the reported bug)', async () => {
    // Regression for "didn't auto-scroll all the time": content grows (a new
    // final row / live partial) but the browser leaves scrollTop where it was,
    // then a scroll echo fires. The old rAF-guarded code re-judged geometry
    // (now short of the taller bottom) and latched follow-tail OFF. Follow must
    // survive: the echo lands on the value we last snapped to → consumed.
    const { container, rerender } = render(TranscriptPane, { props: baseProps(20) });
    const [left, right] = Array.from(container.querySelectorAll('.overflow-auto')) as HTMLElement[];
    geom(left!, 1000); geom(right!, 500);
    await rerender(baseProps(21)); // growth → snap pins both to bottom
    await raf(); await raf();
    expect(left!.scrollTop).toBe(1000); // engaged

    // Content grew (1000 → 1200) WITHOUT the browser moving scrollTop, then the
    // deferred scroll event for the earlier snap fires at the stale position.
    geom(left!, 1200);
    left!.dispatchEvent(new Event('scroll')); // scrollTop 1000 === last snap → echo
    await rerender(baseProps(22));
    await raf(); await raf();
    expect(left!.scrollTop).toBe(1200); // still following → snapped to new bottom
  });

  it('re-engages follow-tail when the transcript resets (new session)', async () => {
    // Regression: `follow` survived across sessions because the live pane is
    // not remounted — scroll up once, start a NEW session, and auto-scroll
    // was dead from the first word until a manual scroll to the bottom.
    const { container, rerender } = render(TranscriptPane, { props: baseProps(20) });
    const [left, right] = Array.from(container.querySelectorAll('.overflow-auto')) as HTMLElement[];
    geom(left!, 1000); geom(right!, 500);

    // User scrolls away from the bottom → disengage.
    left!.scrollTop = 500;
    left!.dispatchEvent(new Event('scroll'));

    // Session reset: both transcripts empty (store.reset on start).
    await rerender({ ...baseProps(0), translation: [] });
    // New session content arrives — a fresh transcript must follow its tail.
    await rerender(baseProps(21));
    await raf(); await raf();
    expect(left!.scrollTop).toBe(1000);
  });

  it('re-engaging at one column snaps the OTHER to its own bottom immediately', async () => {
    // Regression: re-engaging via the short column merely mirrored its pixel
    // offset onto the tall column (mid-history) and nothing corrected it until
    // the NEXT growth — seconds of "auto-scroll still broken" in a silence.
    const { container, rerender } = render(TranscriptPane, { props: baseProps(20) });
    const [left, right] = Array.from(container.querySelectorAll('.overflow-auto')) as HTMLElement[];
    geom(left!, 1000); geom(right!, 500);
    await rerender(baseProps(21));
    await raf(); await raf();
    expect(left!.scrollTop).toBe(1000); // engaged

    // Disengage from the tall column.
    left!.scrollTop = 300;
    left!.dispatchEvent(new Event('scroll'));
    right!.dispatchEvent(new Event('scroll')); // mirror echo, consumed

    // User returns to the SHORT column's bottom → re-engage. Both columns
    // must snap to their own bottoms now, not wait for the next token.
    right!.scrollTop = 480; // 480+100 >= 500-32
    right!.dispatchEvent(new Event('scroll'));
    await raf(); await raf();
    expect(left!.scrollTop).toBe(1000);
    expect(right!.scrollTop).toBe(500);
  });

  it('shows a Jump-to-latest pill while disengaged; clicking it re-engages', async () => {
    // Re-engaging by scroll is a moving-target game (the bottom recedes every
    // ~100 ms while captions stream); the pill is the deterministic resume.
    const { container, queryByText, getByText, rerender } = render(TranscriptPane, { props: baseProps(20) });
    const [left, right] = Array.from(container.querySelectorAll('.overflow-auto')) as HTMLElement[];
    geom(left!, 1000); geom(right!, 500);
    await rerender(baseProps(21));
    await raf(); await raf();
    expect(queryByText(/Jump to latest/)).toBeNull(); // following → no pill

    left!.scrollTop = 300;
    left!.dispatchEvent(new Event('scroll'));
    await raf();
    const pill = getByText(/Jump to latest/); // disengaged → pill visible

    await fireEvent.click(pill);
    await raf(); await raf();
    expect(left!.scrollTop).toBe(1000);
    expect(right!.scrollTop).toBe(500);
    await waitFor(() => expect(queryByText(/Jump to latest/)).toBeNull());
  });

  it('re-engages from the right column; mirror echoes never override the user column', async () => {
    const { container, rerender } = render(TranscriptPane, { props: baseProps(20) });
    const [left, right] = Array.from(container.querySelectorAll('.overflow-auto')) as HTMLElement[];
    // Fake geometry: TALL left (1000), SHORT right (500), 100px viewports.
    geom(left!, 1000); geom(right!, 500);
    await rerender(baseProps(21)); // growth → snap pins both to their bottoms
    await raf(); await raf();
    expect(left!.scrollTop).toBe(1000); // engaged

    // User scrolls the TALL column up, away from its bottom → disengage. The
    // handler mirrors that position onto the SHORT column.
    left!.scrollTop = 500; // 500+100 < 1000-32
    left!.dispatchEvent(new Event('scroll'));
    // The mirror's echo on the short column fires next — it lands on the value
    // we just wrote there, so it's consumed and must NOT re-engage follow-tail.
    right!.dispatchEvent(new Event('scroll'));

    await rerender(baseProps(22)); // growth: would snap if (wrongly) engaged
    await raf(); await raf();
    expect(left!.scrollTop).toBe(500); // no snap → still disengaged

    // Now the USER scrolls the right column to its bottom → re-engage.
    right!.scrollTop = 480; // 480+100 >= 500-32
    right!.dispatchEvent(new Event('scroll'));
    await rerender(baseProps(23));
    await raf(); await raf();
    expect(left!.scrollTop).toBe(1000); // snapped → re-engaged from the right
  });
});
