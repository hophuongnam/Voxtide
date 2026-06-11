<script lang="ts">
  import { toPinyin, type PinyinChar } from '../../lib/pinyin';

  interface Props {
    text: string;
    /** Live partials re-render every frame; converting the whole line each
     *  time defeats the pinyin cache (the text never repeats). Debounce the
     *  conversion 150 ms and render the un-converted tail plain meanwhile —
     *  finalized text (live=false) converts immediately. */
    live?: boolean;
  }
  const { text, live = false }: Props = $props();

  let debounced = $state('');
  $effect(() => {
    if (!live) return;
    const current = text;
    const t = setTimeout(() => {
      debounced = current;
    }, 150);
    return () => clearTimeout(t);
  });

  // The converted portion: full text when finalized; the last debounced
  // prefix while live (empty until the first debounce fires, or after a
  // partial REWRITE where the old conversion would mislead).
  const stable = $derived.by(() => {
    if (!live) return text;
    return text.startsWith(debounced) ? debounced : '';
  });
  // Whatever isn't converted yet renders plain; pinyin lands ≤150 ms later.
  const tail = $derived(live ? text.slice(stable.length) : '');

  type Segment =
    | { kind: 'plain'; text: string }
    | { kind: 'ruby'; char: string; pinyin: string };

  const segments = $derived.by<Segment[]>(() => {
    let chars: PinyinChar[];
    try {
      chars = toPinyin(stable);
    } catch {
      return stable ? [{ kind: 'plain', text: stable }] : [];
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

<!-- Keep this on ONE line: line breaks become whitespace text nodes that
     visually separate adjacent ruby characters. Do not reformat/split. -->
{#each segments as s}{#if s.kind === 'plain'}{s.text}{:else}<ruby>{s.char}<rt>{s.pinyin}</rt></ruby>{/if}{/each}{tail}

<style>
  ruby { ruby-position: over; }
  rt {
    font-size: 0.5em;
    line-height: 1;
    color: var(--vt-muted);
  }
</style>
