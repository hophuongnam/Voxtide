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
  // $state: drives the Jump-to-latest pill, and makes the follow-tail effect
  // re-run on re-engage (snap immediately, not on the next token).
  let follow = $state(true);
  const NEAR_BOTTOM_PX = 32;

  // Echo suppression is value-based, not timing-based. Every programmatic
  // scroll (follow-tail snap or column mirror) records its exact target and
  // arms a one-shot: the next scroll event landing on that value is our own
  // echo and is consumed; anything else is the user. Each event disarms the
  // slot, so a stale target can't keep eating the user's later scrolls back to
  // the bottom. The old rAF-window guards raced under rapid live partials —
  // an echo firing just after its guard cleared latched follow-tail off, so
  // the transcript "didn't auto-scroll all the time".
  const prog = new WeakMap<HTMLElement, number>();
  function setScroll(el: HTMLElement, top: number) {
    el.scrollTop = top;
    prog.set(el, el.scrollTop); // read back post-clamp
  }
  function isEcho(el: HTMLElement): boolean {
    const p = prog.get(el);
    prog.delete(el); // disarm on every event — user or echo
    return p !== undefined && Math.abs(el.scrollTop - p) < 2;
  }
  const nearBottom = (el: HTMLElement) =>
    el.scrollTop + el.clientHeight >= el.scrollHeight - NEAR_BOTTOM_PX;

  $effect(() => {
    if (!leftEl || !rightEl) return;
    const l = leftEl, r = rightEl;
    // A genuine user scroll sets follow intent from THAT column's own geometry
    // (heights differ, so either column must be able to re-engage), then
    // mirrors to the other so the two columns browse history together.
    const handler = (self: HTMLElement, other: HTMLElement) => () => {
      if (isEcho(self)) return;
      follow = nearBottom(self);
      // Browsing history mirrors the columns pixel-for-pixel; re-engaging
      // hands BOTH columns to the follow-tail effect instead (a mirror here
      // would strand the other, differently-tall column at this column's
      // offset until the next token happened to arrive).
      if (!follow) setScroll(other, self.scrollTop);
    };
    const onL = handler(l, r), onR = handler(r, l);
    l.addEventListener('scroll', onL, { passive: true });
    r.addEventListener('scroll', onR, { passive: true });
    return () => {
      l.removeEventListener('scroll', onL);
      r.removeEventListener('scroll', onR);
    };
  });

  // Follow-tail: when transcript grows AND the user is still following, snap
  // both columns to their own bottoms. Reads content lengths so Svelte re-runs
  // this on any growth; one rAF lets layout settle so scrollHeight is final.
  // Also re-runs when `follow` flips true (re-engage/pill) so the snap is
  // immediate rather than deferred to the next token.
  $effect(() => {
    // An empty transcript is a new/reset session: always follow its tail.
    // `follow` otherwise survives the reset (the live pane is not remounted
    // between sessions), leaving auto-scroll dead from the first word.
    if (original.length + translation.length + liveOriginal.length + liveTranslation.length === 0) {
      follow = true;
    }
    if (!follow || !leftEl || !rightEl) return;
    const l = leftEl, r = rightEl;
    requestAnimationFrame(() => {
      if (!follow) return; // user scrolled up between growth and this frame
      setScroll(l, l.scrollHeight);
      setScroll(r, r.scrollHeight);
    });
  });
</script>

<div class="flex-1 flex overflow-hidden relative" data-testid="transcript-root"
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
  <!-- Deterministic resume: re-engaging by scroll means landing within 32px of
       a bottom that recedes with every token — a moving-target game that
       "sometimes" fails. The pill both signals that follow-tail is off and
       re-engages it in one click (the effect above snaps on follow=true). -->
  {#if !follow}
    <button class="absolute bottom-3 left-1/2 -translate-x-1/2 z-10 px-3 py-1 rounded-full text-xs font-medium shadow-md cursor-pointer"
            style:background="var(--vt-accent)" style:color="var(--vt-bg)"
            style:border="0.5px solid var(--vt-accent-tint-25)"
            onclick={() => (follow = true)}>
      Jump to latest ↓
    </button>
  {/if}
</div>
