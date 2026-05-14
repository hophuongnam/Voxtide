<script lang="ts">
  import { onMount } from 'svelte';
  import OverlayWindow from '../components/overlay/OverlayWindow.svelte';
  import { hideOverlay, onCoreEvent } from '../lib/ipc';
  import type { CoreEvent } from '../lib/ipc';

  // Finalized sentences (capped at the last 5; OverlayWindow only renders that many).
  let lines = $state<string[]>([]);
  // The current sentence still being built from final tokens (Soniox emits one final per
  // sub-word). Flushed into `lines` on sentence-end punctuation or speaker change.
  let pending = $state<string>('');
  let pendingChip = $state<string | null>(null);
  // Frame-level non-final partial (cumulative trailing tokens) — overwritten each Live event.
  let live = $state<string>('');

  let connState = $state<'active' | 'reconnecting' | 'idle'>('idle');
  let attempt = $state<number | null>(null);
  let retryInMs = $state<number | null>(null);
  let connectionLabel = $state('IDLE');
  let hover = $state(false);

  const SENTENCE_END = /[.!?。！？]\s*$/;

  function flushPending() {
    if (!pending) return;
    lines = [...lines, pending].slice(-5);
    pending = '';
  }

  function handle(ev: CoreEvent) {
    switch (ev.kind) {
      case 'transcript-live':
        if (ev.status === 'translation') live = ev.text;
        break;
      case 'transcript-final':
        if (ev.status !== 'translation') break;
        // Speaker change → previous sentence is done, regardless of punctuation.
        if (pendingChip !== null && ev.chip !== pendingChip) flushPending();
        pending = pending + ev.text;
        pendingChip = ev.chip;
        // The non-final partial only contains tokens past the cursor — once a token
        // finalizes it disappears from `live`, so clear our local copy too.
        live = '';
        if (SENTENCE_END.test(pending)) flushPending();
        break;
      case 'connection-state':
        connState = ev.state;
        attempt = ev.attempt;
        retryInMs = ev.retry_in_ms;
        if (connState === 'active') connectionLabel = 'EN → VI';
        break;
      case 'session-started':
        connState = 'active';
        lines = []; pending = ''; pendingChip = null; live = '';
        break;
      case 'session-stopped':
        flushPending();
        connState = 'idle';
        pendingChip = null;
        live = '';
        break;
    }
  }

  // Bottom row = pending (already-finalized tokens of current sentence) + live (partial).
  // Both can be present simultaneously; concatenation is correct because Soniox removes a
  // token from the non-final stream as soon as it finalizes.
  const displayLines = $derived.by(() => {
    const bottom = pending + live;
    return bottom ? [...lines.slice(-4), bottom] : lines;
  });

  let hoverTimer: ReturnType<typeof setTimeout> | undefined;
  function onEnter() {
    hover = true;
    if (hoverTimer) clearTimeout(hoverTimer);
  }
  function onLeave() {
    if (hoverTimer) clearTimeout(hoverTimer);
    hoverTimer = setTimeout(() => { hover = false; }, 1500);
  }

  onMount(() => {
    let unlisten: (() => void) | undefined;
    onCoreEvent(handle).then((un) => { unlisten = un; });
    return () => { if (unlisten) unlisten(); };
  });
</script>

<div class="w-screen h-screen" role="presentation" onmouseenter={onEnter} onmouseleave={onLeave}>
  <OverlayWindow
    lines={displayLines}
    state={connState}
    {connectionLabel}
    {hover}
    attempt={attempt ?? 1}
    retryInMs={retryInMs ?? 0}
    onclose={() => hideOverlay()} />
</div>
