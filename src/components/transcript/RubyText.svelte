<script lang="ts">
  import { toPinyin } from '../../lib/pinyin';

  interface Props { text: string }
  const { text }: Props = $props();

  type Segment =
    | { kind: 'plain'; text: string }
    | { kind: 'ruby'; char: string; pinyin: string };

  const segments = $derived.by<Segment[]>(() => {
    let chars;
    try {
      chars = toPinyin(text);
    } catch {
      return text ? [{ kind: 'plain', text }] : [];
    }
    const out: Segment[] = [];
    for (const c of chars) {
      if (c.pinyin === '') {
        const last = out[out.length - 1];
        if (last && last.kind === 'plain') last.text += c.char;
        else out.push({ kind: 'plain', text: c.char });
      } else {
        out.push({ kind: 'ruby', char: c.char, pinyin: c.pinyin });
      }
    }
    return out;
  });
</script>

{#each segments as s}{#if s.kind === 'plain'}{s.text}{:else}<ruby>{s.char}<rt>{s.pinyin}</rt></ruby>{/if}{/each}

<style>
  ruby { ruby-position: over; }
  rt {
    font-size: 0.5em;
    line-height: 1;
    color: var(--vt-muted);
  }
</style>
