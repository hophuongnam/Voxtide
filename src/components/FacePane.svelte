<script lang="ts">
  import type { TranscriptLine } from '../types';

  interface LivePartial { text: string; translated: boolean; }
  interface Props {
    lines: TranscriptLine[];
    live: LivePartial[];
    /** Far pane is rotated 180° so it reads right-side-up for the person across
     *  the table. Rotation is paint-only — scrollTop/scrollHeight stay in DOM
     *  coords, so the follow-tail math below is identical for both panes. */
    rotated?: boolean;
    /** Auto-snap to the newest line as content grows (live capture). False for
     *  replay of a past session, so it opens at the START (oldest) for reading. */
    follow?: boolean;
  }
  const { lines, live, rotated = false, follow = true }: Props = $props();

  let el = $state<HTMLElement | null>(null);
  let atBottom = true;
  const NEAR_BOTTOM_PX = 32;

  // Total rendered text length — grows on both array append AND in-place text
  // growth (appendFinal mutates the last row's text without changing length),
  // so the follow-tail effect re-runs on every kind of content change.
  const contentLen = $derived(
    lines.reduce((n, l) => n + l.text.length, 0) + live.reduce((n, l) => n + l.text.length, 0),
  );

  $effect(() => {
    const e = el;
    if (!e) return;
    const onScroll = () => {
      atBottom = e.scrollTop + e.clientHeight >= e.scrollHeight - NEAR_BOTTOM_PX;
    };
    e.addEventListener('scroll', onScroll, { passive: true });
    return () => e.removeEventListener('scroll', onScroll);
  });

  // Follow-tail: snap to newest when content grows and the user was at bottom.
  // rAF lets layout settle so scrollHeight is current before the snap.
  $effect(() => {
    contentLen; // reactive dep
    const e = el;
    if (!follow || !atBottom || !e) return;
    requestAnimationFrame(() => { e.scrollTop = e.scrollHeight; });
  });
</script>

<div class="pane" class:rot={rotated} bind:this={el}>
  {#each lines as l}
    <p class="ln" class:tr={l.status === 'translation'}>
      {#if l.chip}<b class="chip">{l.chip}</b>{/if}{l.text}
    </p>
  {/each}
  {#each live as lv}
    <p class="ln live" class:tr={lv.translated}>{lv.text}</p>
  {/each}
</div>

<style>
  .pane { flex: 1; min-height: 0; overflow-y: auto; padding: 12px 16px; }
  /* Far pane: 180° so it faces the person across the table. */
  .rot { transform: rotate(180deg); }
  .ln { margin: 6px 0; font-size: 19px; line-height: 1.35; color: var(--vt-text); }
  /* Functional cue (not role-based): translated text is accented so each reader
     can tell their own speech apart from the translation of the other person. */
  .ln.tr { color: var(--vt-accent); }
  .ln.live { opacity: 0.55; }
  .chip {
    display: inline-block; min-width: 1.4em; margin-right: 6px;
    color: var(--vt-muted); font-weight: 700;
  }
</style>
