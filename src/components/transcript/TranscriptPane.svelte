<script lang="ts">
  import Column from './Column.svelte';
  import Line from './Line.svelte';
  import type { Mode, TranscriptLine, FontSize, AppConfig } from '../../types';

  interface Props {
    mode: Mode;
    a: { code: string; name: string };
    b: { code: string; name: string };
    original: TranscriptLine[];
    translation: TranscriptLine[];
    liveOriginal: string;
    liveTranslation: string;
    /** Detected language of the in-flight partials (from the wire event);
     *  the column code is only the fallback. */
    liveOriginalLang?: string | null;
    liveTranslationLang?: string | null;
    fontSize?: FontSize;
    showPinyin?: boolean;
    /** System Audio session with the local mic blended in → runs two-way, so
     *  the column headers reflect bidirectional content (same as Conversation).
     *  Only meaningful for the live view; replay of past sessions passes false. */
    captureMic?: boolean;
    cfg?: AppConfig | null;
    onconfigchange?: (next: AppConfig) => void;
  }
  const {
    mode, a, b, original, translation, liveOriginal, liveTranslation,
    liveOriginalLang = null, liveTranslationLang = null,
    fontSize = 'm', showPinyin = false, captureMic = false, cfg = null, onconfigchange = () => {},
  }: Props = $props();

  // Two-way whenever both languages share one stream: Conversation always, and
  // System Audio when the local mic is blended in. Drives the column headers;
  // placement itself is status-based in the store and never branches on mode.
  const twoWay = $derived(mode === 'conversation' || (mode === 'meeting' && captureMic));

  // The `m` value must stay in sync with Line.svelte's var(--vt-transcript-size, 13.5px) fallback.
  const FONT_PX: Record<FontSize, string> = {
    xs: '11px', s: '12.5px', m: '13.5px', l: '16px', xl: '19px',
  };

  const originalCode = $derived(!twoWay
    ? `${a.code} · multi-speaker`
    : `${a.code}/${b.code}`);
  const translationCode = $derived(!twoWay
    ? b.code
    : `${a.code} ⇄ ${b.code}`);
  const translationSub = $derived(!twoWay ? 'target' : 'two-way');

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
    const nearBottom = (el: HTMLElement) =>
      el.scrollTop + el.clientHeight >= el.scrollHeight - NEAR_BOTTOM_PX;
    // Each column judges follow-tail from its OWN geometry, but only for USER
    // scrolls: `syncing` marks mirror echoes, which must never vote — the
    // mirror clamps the shorter column near its own bottom, so an echo there
    // would re-engage while the user is scrolling AWAY in the taller one.
    // (Judging only the left column was the old bug: with a taller left,
    // bottoming out the right could never re-engage.)
    const onL = () => {
      if (!syncing && !autoScrolling) atBottom = nearBottom(l);
      mirror(l, r);
    };
    const onR = () => {
      if (!syncing && !autoScrolling) atBottom = nearBottom(r);
      mirror(r, l);
    };
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

<div class="flex-1 flex overflow-hidden" data-testid="transcript-root"
     style:--vt-transcript-size={FONT_PX[fontSize]}>
  <Column label="Original" code={originalCode} sub={mode === 'meeting' ? 'diarized' : 'per turn'}
          bodyRef={(el) => leftEl = el} cfg={cfg} onconfigchange={onconfigchange}>
    {#each original as line}
      <Line {line} {showPinyin} />
    {/each}
    <!-- Live-line language: the DETECTED language from the wire event when
         present, else the column's 2-letter ISO code. The pinyin gate in
         Line.svelte matches `language === 'zh'`, so live zh gets pinyin
         immediately instead of reflowing when the line finalizes. -->
    {#if liveOriginal}
      <Line line={{ ts_ms: Date.now(), status: 'original', text: liveOriginal,
                    language: liveOriginalLang ?? a.code.toLowerCase(), chip: null, live: true }} {showPinyin} />
    {/if}
  </Column>
  <div class="w-px" style:background="var(--vt-border)"></div>
  <Column label="Translation" code={translationCode} sub={translationSub} accent
          bodyRef={(el) => rightEl = el} cfg={cfg} onconfigchange={onconfigchange}>
    {#each translation as line}
      <Line {line} {showPinyin} translated />
    {/each}
    {#if liveTranslation}
      <Line line={{ ts_ms: Date.now(), status: 'translation', text: liveTranslation,
                    language: liveTranslationLang ?? b.code.toLowerCase(), chip: null, live: true }} {showPinyin} translated />
    {/if}
  </Column>
</div>
