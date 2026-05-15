import { pinyin } from 'pinyin-pro';

export interface PinyinChar {
  char: string;
  pinyin: string;
}

// pinyin-pro `type: 'all'` returns one record per codepoint:
//   { origin: string; pinyin: string; isZh: boolean; ... }
// For non-Chinese codepoints isZh is false and pinyin echoes origin.
// For an unresolvable Han codepoint pinyin also echoes origin — treat
// any "no real pinyin" case as plain text (empty string).
interface PinyinRecord {
  origin: string;
  pinyin: string;
  isZh: boolean;
}

const CACHE_MAX = 256;
const cache = new Map<string, PinyinChar[]>();

function compute(text: string): PinyinChar[] {
  const records = pinyin(text, {
    type: 'all',
    toneType: 'symbol',
  }) as unknown as PinyinRecord[];

  return records.map((r) => {
    const real = r.isZh && r.pinyin && r.pinyin !== r.origin;
    return { char: r.origin, pinyin: real ? r.pinyin : '' };
  });
}

export function toPinyin(text: string): PinyinChar[] {
  if (!text) return [];

  const hit = cache.get(text);
  if (hit) {
    // Refresh recency: delete + re-insert moves the key to newest.
    cache.delete(text);
    cache.set(text, hit);
    return hit;
  }

  let result: PinyinChar[];
  try {
    result = compute(text);
  } catch {
    // Never let a pinyin failure make the transcript unreadable.
    result = [{ char: text, pinyin: '' }];
  }

  cache.set(text, result);
  if (cache.size > CACHE_MAX) {
    const oldest = cache.keys().next().value as string;
    cache.delete(oldest);
  }
  return result;
}
