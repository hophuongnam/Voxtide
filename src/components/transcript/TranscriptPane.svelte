<script lang="ts">
  import Column from './Column.svelte';
  import Line from './Line.svelte';
  import type { Mode, TranscriptLine, WhichLang } from '../../types';

  interface Props {
    mode: Mode;
    a: { code: string; name: string };
    b: { code: string; name: string };
    mine: WhichLang;
    original: TranscriptLine[];
    translation: TranscriptLine[];
    liveOriginal: string;
    liveTranslation: string;
  }
  const { mode, a, b, mine, original, translation, liveOriginal, liveTranslation }: Props = $props();

  const originalCode = $derived(mode === 'meeting'
    ? `${a.code} · multi-speaker`
    : `${a.code}/${b.code}`);
  const translationCode = $derived(mode === 'meeting'
    ? (mine === 'a' ? a.code : b.code)
    : `${a.code} ⇄ ${b.code}`);
  const translationSub = $derived(mode === 'meeting' ? 'target' : 'two-way');
</script>

<div class="flex-1 flex overflow-hidden">
  <Column label="Original" code={originalCode} sub={mode === 'meeting' ? 'diarized' : 'per turn'}>
    {#each original as line}
      <Line {line} />
    {/each}
    {#if liveOriginal}
      <Line line={{ ts_ms: Date.now(), status: 'original', text: liveOriginal,
                    language: a.code.toLowerCase(), chip: null, live: true }} />
    {/if}
  </Column>
  <div class="w-px" style:background="var(--vt-border)"></div>
  <Column label="Translation" code={translationCode} sub={translationSub} accent>
    {#each translation as line}
      <Line {line} translated />
    {/each}
    {#if liveTranslation}
      <Line line={{ ts_ms: Date.now(), status: 'translation', text: liveTranslation,
                    language: b.code.toLowerCase(), chip: null, live: true }} translated />
    {/if}
  </Column>
</div>
