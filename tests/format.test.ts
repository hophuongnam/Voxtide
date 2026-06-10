import { describe, it, expect } from 'vitest';
import { groupByDate, formatDuration, formatTime, formatHotkey } from '../src/lib/format';

describe('groupByDate', () => {
  // Anchor everything in local time so the test is timezone-agnostic.
  const now = new Date(2026, 4, 13, 15, 0, 0).getTime();

  it('classifies relative buckets', () => {
    const items = [
      { id: 1, t: new Date(2026, 4, 13,  8, 0, 0).getTime() },  // Today
      { id: 2, t: new Date(2026, 4, 12, 20, 0, 0).getTime() },  // Yesterday
      { id: 3, t: new Date(2026, 4,  9, 12, 0, 0).getTime() },  // This week
      { id: 4, t: new Date(2026, 3, 22, 12, 0, 0).getTime() },  // Earlier
    ];
    const groups = groupByDate(items, x => x.t, now);
    expect(groups.map(g => g.label)).toEqual(['Today', 'Yesterday', 'This week', 'Earlier']);
    expect(groups[0]!.items).toHaveLength(1);
  });
});

describe('formatDuration', () => {
  it('formats ms to compact strings', () => {
    expect(formatDuration(0)).toBe('—');
    expect(formatDuration(38_000)).toBe('38s');
    expect(formatDuration(240_000)).toBe('4m');
    expect(formatDuration(3_840_000)).toBe('1h 04m');
  });
});

describe('formatTime', () => {
  it('renders short clock time', () => {
    const t = new Date('2026-05-13T14:22:00').getTime();
    expect(formatTime(t)).toMatch(/14:22|2:22/);
  });
});

describe('formatHotkey', () => {
  const setPlatform = (value: string) =>
    Object.defineProperty(window.navigator, 'platform', { value, configurable: true });

  it('renders mac glyphs joined without separators', () => {
    setPlatform('MacIntel');
    expect(formatHotkey('CommandOrControl+Shift+V')).toBe('⌘⇧V');
    expect(formatHotkey('Control+Shift+V')).toBe('⌃⇧V');
    expect(formatHotkey('Alt+x')).toBe('⌥X');
  });

  it('renders plus-joined names elsewhere', () => {
    setPlatform('Win32');
    expect(formatHotkey('CommandOrControl+Shift+V')).toBe('Ctrl+Shift+V');
    expect(formatHotkey('Alt+F5')).toBe('Alt+F5');
  });

  it('passes through unknown keys uppercased', () => {
    setPlatform('MacIntel');
    expect(formatHotkey('F9')).toBe('F9');
  });
});
