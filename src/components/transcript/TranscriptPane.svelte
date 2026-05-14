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

  let leftEl: HTMLElement | null = $state(null);
  let rightEl: HTMLElement | null = $state(null);
  let syncing = false;
  let autoScrolling = false;
  let atBottom = true;
  const NEAR_BOTTOM_PX = 32;

  function snapBottom(el: HTMLElement | null) {
    if (!el) return;
    el.scrollTop = el.scrollHeight;
  }

  $effect(() => {
    if (!leftEl || !rightEl) return;
    const l = leftEl, r = rightEl;
    const mirror = (from: HTMLElement, to: HTMLElement) => {
      // Skip mirroring while auto-scroll has each column independently
      // pinned to its own bottom — otherwise the mirror clobbers the
      // explicit `to.scrollTop = to.scrollHeight` when the two columns
      // have different scrollHeights.
      if (syncing || autoScrolling) return;
      syncing = true;
      to.scrollTop = from.scrollTop;
      requestAnimationFrame(() => { syncing = false; });
    };
    const recomputeAtBottom = () => {
      if (autoScrolling) return;
      atBottom = l.scrollTop + l.clientHeight >= l.scrollHeight - NEAR_BOTTOM_PX;
    };
    const onL = () => { mirror(l, r); recomputeAtBottom(); };
    const onR = () => { mirror(r, l); };
    l.addEventListener('scroll', onL, { passive: true });
    r.addEventListener('scroll', onR, { passive: true });
    return () => {
      l.removeEventListener('scroll', onL);
      r.removeEventListener('scroll', onR);
    };
  });

  // Follow-tail: when transcript grows AND user was at bottom, snap to bottom.
  // Reads content lengths so Svelte re-runs this on any growth.
  $effect(() => {
    // Track every content surface as a reactive dep.
    original.length; translation.length;
    liveOriginal.length; liveTranslation.length;
    if (!atBottom || !leftEl || !rightEl) return;
    autoScrolling = true;
    // Double-rAF: first frame lets the browser apply the new DOM layout so
    // scrollHeight is settled; second frame releases the autoScrolling guard
    // after the snap-induced scroll events have drained.
    const l = leftEl, r = rightEl;
    requestAnimationFrame(() => {
      snapBottom(l); snapBottom(r);
      requestAnimationFrame(() => { autoScrolling = false; });
    });
  });
</script>

<div class="flex-1 flex overflow-hidden">
  <Column label="Original" code={originalCode} sub={mode === 'meeting' ? 'diarized' : 'per turn'}
          bodyRef={(el) => leftEl = el}>
    {#each original as line}
      <Line {line} />
    {/each}
    {#if liveOriginal}
      <Line line={{ ts_ms: Date.now(), status: 'original', text: liveOriginal,
                    language: a.code.toLowerCase(), chip: null, live: true }} />
    {/if}
  </Column>
  <div class="w-px" style:background="var(--vt-border)"></div>
  <Column label="Translation" code={translationCode} sub={translationSub} accent
          bodyRef={(el) => rightEl = el}>
    {#each translation as line}
      <Line {line} translated />
    {/each}
    {#if liveTranslation}
      <Line line={{ ts_ms: Date.now(), status: 'translation', text: liveTranslation,
                    language: b.code.toLowerCase(), chip: null, live: true }} translated />
    {/if}
  </Column>
</div>
