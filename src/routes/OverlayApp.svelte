<script lang="ts">
  import { onMount } from 'svelte';
  import OverlayWindow from '../components/overlay/OverlayWindow.svelte';
  import { getConfig, hideOverlay, onConfigChanged, onCoreEvent } from '../lib/ipc';
  import { formatHotkey } from '../lib/format';
  import { DEFAULT_HOTKEY } from '../lib/modes';
  import { applyTheme } from '../theme/theme';
  import type { CoreEvent } from '../lib/ipc';
  import type { AppConfig } from '../types';

  // Finalized utterances (capped at the last 5; OverlayWindow only renders that many).
  let lines = $state<string[]>([]);
  // The current utterance still being built from final tokens (Soniox emits one final per
  // sub-word). Flushed into `lines` on an utterance break or speaker change.
  let pending = $state<string>('');
  let pendingChip = $state<string | null>(null);
  // Frame-level non-final partial (cumulative trailing tokens) — overwritten each Live event.
  let live = $state<string>('');

  let connState = $state<'active' | 'reconnecting' | 'idle'>('idle');
  let attempt = $state<number | null>(null);
  let retryInMs = $state<number | null>(null);
  let hover = $state(false);

  // This webview has its own config copy: labels, hotkey hint and theme all
  // derive from it (they were hardcoded 'EN → VI' / ⌃⇧V / dark before).
  let cfg = $state<AppConfig | null>(null);
  const pairLabel = $derived.by(() => {
    if (!cfg) return '';
    const a = cfg.language_a.toUpperCase();
    const b = cfg.language_b.toUpperCase();
    return cfg.mode === 'meeting' ? `${a} → ${b}` : `${a} ⇄ ${b}`;
  });
  const hotkeyLabel = $derived(formatHotkey(cfg?.hotkey ?? DEFAULT_HOTKEY));
  function adoptConfig(c: AppConfig) {
    cfg = c;
    // applyTheme mutates THIS document's body — the overlay window themes
    // itself (it was pinned dark; light tokens were unreachable).
    applyTheme(c.theme);
  }

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
        // Speaker change → previous utterance is done, regardless of pauses.
        if (pendingChip !== null && ev.chip !== pendingChip) flushPending();
        pending = pending + ev.text;
        pendingChip = ev.chip;
        // The non-final partial only contains tokens past the cursor — once a token
        // finalizes it disappears from `live`, so clear our local copy too.
        live = '';
        break;
      case 'utterance-break':
        // Speech pause — the same row boundary the main transcript uses. We
        // deliberately never break on punctuation (ASCII `.!?` vs CJK `。！？`
        // tokenize asymmetrically across languages); without this case a
        // punctuation-less target language never flushed under the 5-line cap.
        flushPending();
        break;
      case 'connection-state':
        connState = ev.state;
        attempt = ev.attempt;
        retryInMs = ev.retry_in_ms;
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
    let unConfig: (() => void) | undefined;
    onCoreEvent(handle).then((un) => { unlisten = un; });
    // Guarded like other Tauri calls in onMount: vitest/jsdom has no runtime.
    if ((window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__) {
      getConfig().then(adoptConfig).catch((e) => console.debug('overlay config load failed', e));
      onConfigChanged(adoptConfig).then((un) => { unConfig = un; });
    }
    return () => { unlisten?.(); unConfig?.(); };
  });
</script>

<div class="w-screen h-screen" role="presentation" onmouseenter={onEnter} onmouseleave={onLeave}>
  <OverlayWindow
    lines={displayLines}
    state={connState}
    connectionLabel={pairLabel}
    {hover}
    {hotkeyLabel}
    attempt={attempt ?? 1}
    retryInMs={retryInMs ?? 0}
    onclose={() => hideOverlay()} />
</div>
