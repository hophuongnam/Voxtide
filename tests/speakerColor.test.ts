import { describe, it, expect } from 'vitest';
import { speakerVar } from '../src/lib/speakerColor';

describe('speakerVar', () => {
  it('returns the right CSS var per letter', () => {
    expect(speakerVar('A')).toBe('var(--vt-speaker-a)');
    expect(speakerVar('B')).toBe('var(--vt-speaker-b)');
    expect(speakerVar('C')).toBe('var(--vt-speaker-c)');
    expect(speakerVar('D')).toBe('var(--vt-speaker-d)');
  });
  it('wraps at E back to A', () => {
    expect(speakerVar('E')).toBe('var(--vt-speaker-a)');
  });
});
