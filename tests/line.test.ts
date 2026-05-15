import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/svelte';
import Line from '../src/components/transcript/Line.svelte';

const base = { ts_ms: 0, status: 'original' as const, chip: null, live: false };

describe('Line pinyin gating', () => {
  it('renders ruby when showPinyin and language is zh', () => {
    const { container } = render(Line, {
      props: { line: { ...base, text: '你好', language: 'zh' }, showPinyin: true },
    });
    expect(container.querySelectorAll('ruby').length).toBe(2);
  });

  it('renders plain when showPinyin but language is not zh', () => {
    const { container } = render(Line, {
      props: { line: { ...base, text: 'Hello', language: 'en' }, showPinyin: true },
    });
    expect(container.querySelectorAll('ruby').length).toBe(0);
    expect(container.textContent).toContain('Hello');
  });

  it('renders plain when language is zh but showPinyin is off', () => {
    const { container } = render(Line, {
      props: { line: { ...base, text: '你好', language: 'zh' }, showPinyin: false },
    });
    expect(container.querySelectorAll('ruby').length).toBe(0);
    expect(container.textContent).toContain('你好');
  });

  it('applies the transcript font-size CSS variable', () => {
    const { container } = render(Line, {
      props: { line: { ...base, text: 'Hello', language: 'en' }, showPinyin: false },
    });
    const el = container.querySelector('[data-testid="line-text"]') as HTMLElement;
    expect(el.style.fontSize).toBe('var(--vt-transcript-size, 13.5px)');
  });
});
