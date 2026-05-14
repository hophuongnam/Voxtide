import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/svelte';
import StatusBar from '../src/components/status/StatusBar.svelte';

const props = {
  recording: true, elapsedMs: 38 * 60 * 1000 + 24 * 1000,
  levelDb: -18, latencyMs: 262, mode: 'meeting' as const,
  translationSummary: 'one_way → VI', model: 'stt-rt-v4',
  audioFormat: '16 kHz · mono · s16le',
};

describe('StatusBar visibility breakpoints', () => {
  it('shows all 6 fields at 920px', () => {
    const { container } = render(StatusBar, { props: { ...props, width: 920 } });
    const text = container.textContent ?? '';
    expect(text).toContain('REC');
    expect(text).toContain('00:38:24');
    expect(text).toContain('stt-rt-v4');
    expect(text).toContain('262 ms');
    expect(text).toContain('one_way → VI');
    expect(text).toContain('16 kHz');
  });
  it('hides audio format below 900px', () => {
    const { container } = render(StatusBar, { props: { ...props, width: 720 } });
    expect(container.textContent ?? '').not.toContain('16 kHz');
  });
  it('hides latency below 700px', () => {
    const { container } = render(StatusBar, { props: { ...props, width: 600 } });
    expect(container.textContent ?? '').not.toContain('262 ms');
  });
  it('hides model below 580px', () => {
    const { container } = render(StatusBar, { props: { ...props, width: 480 } });
    expect(container.textContent ?? '').not.toContain('stt-rt-v4');
  });
  it('hides translation summary below 480px', () => {
    const { container } = render(StatusBar, { props: { ...props, width: 460 } });
    expect(container.textContent ?? '').not.toContain('one_way');
  });
});
