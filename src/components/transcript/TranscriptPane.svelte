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
  const p: Props = $props();

  const originalCode = p.mode === 'meeting'
    ? `${p.a.code} · multi-speaker`
    : `${p.a.code}/${p.b.code}`;
  const translationCode = p.mode === 'meeting'
    ? (p.mine === 'a' ? p.a.code : p.b.code)
    : `${p.a.code} ⇄ ${p.b.code}`;
  const translationSub = p.mode === 'meeting' ? 'target' : 'two-way';
</script>

<div class="flex-1 flex overflow-hidden">
  <Column label="Original" code={originalCode} sub={p.mode === 'meeting' ? 'diarized' : 'per turn'}>
    {#each p.original as line}
      <Line {line} />
    {/each}
    {#if p.liveOriginal}
      <Line line={{ ts_ms: Date.now(), status: 'original', text: p.liveOriginal,
                    language: p.a.code.toLowerCase(), chip: null, live: true }} />
    {/if}
  </Column>
  <div class="w-px" style:background="var(--vt-border)"></div>
  <Column label="Translation" code={translationCode} sub={translationSub} accent>
    {#each p.translation as line}
      <Line {line} translated />
    {/each}
    {#if p.liveTranslation}
      <Line line={{ ts_ms: Date.now(), status: 'translation', text: p.liveTranslation,
                    language: p.b.code.toLowerCase(), chip: null, live: true }} translated />
    {/if}
  </Column>
</div>
