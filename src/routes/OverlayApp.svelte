<script lang="ts">
  import { onMount } from 'svelte';
  import OverlayWindow from '../components/overlay/OverlayWindow.svelte';
  import { hideOverlay, onCoreEvent, setOverlayClickThrough } from '../lib/ipc';
  import type { CoreEvent } from '../lib/ipc';

  let lines = $state<string[]>([]);
  let live = $state<string>('');
  let connState = $state<'active' | 'reconnecting' | 'idle'>('idle');
  let attempt = $state<number | null>(null);
  let retryInMs = $state<number | null>(null);
  let connectionLabel = $state('IDLE');
  let hover = $state(false);

  function pushLine(text: string) {
    lines = [...lines, text].slice(-5);
  }

  function handle(ev: CoreEvent) {
    switch (ev.kind) {
      case 'transcript-live':
        if (ev.status === 'translation') live = ev.text;
        break;
      case 'transcript-final':
        if (ev.status === 'translation') {
          pushLine(ev.text);
          live = '';
        }
        break;
      case 'connection-state':
        connState = ev.state;
        attempt = ev.attempt;
        retryInMs = ev.retry_in_ms;
        if (connState === 'active') connectionLabel = 'EN → VI';
        break;
      case 'session-started':
        connState = 'active';
        break;
      case 'session-stopped':
        connState = 'idle';
        lines = [];
        live = '';
        break;
    }
  }

  let hoverTimer: ReturnType<typeof setTimeout> | undefined;
  function onEnter() {
    hover = true;
    setOverlayClickThrough(false).catch(() => {});
    if (hoverTimer) clearTimeout(hoverTimer);
  }
  function onLeave() {
    if (hoverTimer) clearTimeout(hoverTimer);
    hoverTimer = setTimeout(() => {
      hover = false;
      setOverlayClickThrough(true).catch(() => {});
    }, 1500);
  }

  onMount(() => {
    let unlisten: (() => void) | undefined;
    onCoreEvent(handle).then((un) => { unlisten = un; });
    return () => { if (unlisten) unlisten(); };
  });
</script>

<div class="w-screen h-screen" role="presentation" onmouseenter={onEnter} onmouseleave={onLeave}>
  <OverlayWindow
    lines={live ? [...lines.slice(-4), live] : lines}
    state={connState}
    {connectionLabel}
    {hover}
    attempt={attempt ?? 1}
    retryInMs={retryInMs ?? 0}
    onclose={() => hideOverlay()} />
</div>
