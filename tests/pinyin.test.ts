import { describe, it, expect } from 'vitest';
import { toPinyin } from '../src/lib/pinyin';

describe('toPinyin', () => {
  it('maps each Han char to tone-marked pinyin', () => {
    expect(toPinyin('你好')).toEqual([
      { char: '你', pinyin: 'nǐ' },
      { char: '好', pinyin: 'hǎo' },
    ]);
  });

  it('returns empty pinyin for non-Han chars', () => {
    expect(toPinyin('Hi世界')).toEqual([
      { char: 'H', pinyin: '' },
      { char: 'i', pinyin: '' },
      { char: '世', pinyin: 'shì' },
      { char: '界', pinyin: 'jiè' },
    ]);
  });

  it('resolves polyphones by context (银行 → háng not xíng)', () => {
    const r = toPinyin('银行');
    expect(r[1]).toEqual({ char: '行', pinyin: 'háng' });
  });

  it('returns [] for empty input', () => {
    expect(toPinyin('')).toEqual([]);
  });

  it('caches by input string (same reference on repeat)', () => {
    const a = toPinyin('你好世界');
    const b = toPinyin('你好世界');
    expect(b).toBe(a);
  });

  it('evicts LRU entries beyond the cap', () => {
    const first = toPinyin('seed-string');
    for (let i = 0; i < 256; i++) toPinyin('filler-' + i);
    expect(toPinyin('seed-string')).not.toBe(first);
  });
});
