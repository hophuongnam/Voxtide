import { describe, it, expect } from 'vitest';
import { groupByDate, formatDuration, formatTime } from '../src/lib/format';

describe('groupByDate', () => {
  const now = new Date('2026-05-13T15:00:00Z').getTime();
  const ms = (s: string) => new Date(s).getTime();

  it('classifies relative buckets', () => {
    const items = [
      { id: 1, t: ms('2026-05-13T08:00:00Z') },  // Today
      { id: 2, t: ms('2026-05-12T20:00:00Z') },  // Yesterday
      { id: 3, t: ms('2026-05-09T12:00:00Z') },  // This week
      { id: 4, t: ms('2026-04-22T12:00:00Z') },  // Earlier
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
