<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { transcript } from '../lib/stores.svelte';
  import {
    onCoreEvent, startSession, stopSession, getConfig, hasApiKey, setApiKey,
    type CoreEvent,
  } from '../lib/ipc';
  import { startMicCapture, stopMicCapture } from '../lib/miccapture';
  import type { AppConfig } from '../types';

  const ACCOUNT = 'default';
  let cfg = $state<AppConfig | null>(null);
  let hasKey = $state(false);
  let keyInput = $state('');
  let recording = $state(false);
  let err = $state<string | null>(null);
  let unlisten: (() => void) | null = null;

  function handle(ev: CoreEvent) {
    switch (ev.kind) {
      case 'session-started': recording = true; break;
      case 'session-stopped': recording = false; transcript.clearLive(); break;
      case 'transcript-live':
        transcript.live({ status: ev.status, text: ev.text, language: ev.language, chip: ev.chip });
        break;
      case 'transcript-final':
        transcript.final({ status: ev.status, text: ev.text, language: ev.language, chip: ev.chip, ts_ms: ev.ts_ms });
        break;
      case 'utterance-break': transcript.utteranceBreak(); break;
      case 'error': err = ev.message; break;
    }
  }

  async function record() {
    err = null;
    if (!cfg) return;
    try {
      await startSession({
        mode: 'conversation',
        language_a: cfg.language_a,
        language_b: cfg.language_b,
        device_id: '',
        api_key_account: ACCOUNT,
      });
      try {
        await startMicCapture(); // triggers the WebView mic permission prompt
      } catch (e) {
        await stopSession();
        err = 'Mic: ' + (e instanceof Error ? `${e.name} ${e.message}` : String(e));
      }
    } catch (e) {
      err = String(e instanceof Error ? e.message : e);
    }
  }

  async function stop() {
    stopMicCapture();
    await stopSession();
  }

  async function saveKey() {
    if (!keyInput.trim()) return;
    await setApiKey(ACCOUNT, keyInput.trim());
    keyInput = '';
    hasKey = true;
  }

  onMount(async () => {
    // Guard Tauri calls so vitest/jsdom doesn't choke.
    if (typeof window !== 'undefined' && (window as any).__TAURI_INTERNALS__) {
      cfg = await getConfig();
      hasKey = await hasApiKey(ACCOUNT);
      unlisten = await onCoreEvent(handle);
    }
  });
  onDestroy(() => unlisten?.());
</script>

<main class="ff">
  <header>Voxtide — conversation (test)</header>

  {#if !hasKey}
    <div class="setup">
      <input type="password" placeholder="Soniox API key" bind:value={keyInput} />
      <button onclick={saveKey}>Save key</button>
    </div>
  {:else}
    <button class="rec" onclick={() => (recording ? stop() : record())}>
      {recording ? '■ Stop' : '● Record'}
    </button>
  {/if}

  {#if err}<p class="err">{err}</p>{/if}

  <section class="lines">
    {#each transcript.original as l (l.ts_ms + l.text)}
      <p><b>{l.language ?? '?'}:</b> {l.text}</p>
    {/each}
    {#if transcript.liveOriginal}<p class="live">{transcript.liveOriginal}</p>{/if}
    {#each transcript.translation as l (l.ts_ms + l.text)}
      <p class="tr"><b>{l.language ?? '?'}:</b> {l.text}</p>
    {/each}
    {#if transcript.liveTranslation}<p class="live tr">{transcript.liveTranslation}</p>{/if}
  </section>
</main>

<style>
  .ff { padding: 16px; padding-top: calc(env(safe-area-inset-top, 0px) + 16px); font-family: system-ui, sans-serif; }
  header { font-weight: 600; margin-bottom: 12px; }
  .setup { display: flex; gap: 8px; margin-bottom: 12px; }
  .setup input { flex: 1; padding: 10px; }
  .rec { font-size: 20px; padding: 14px 28px; border-radius: 12px; margin-bottom: 12px; }
  .err { color: #c00; }
  .lines p { margin: 4px 0; }
  .tr { color: #06c; }
  .live { opacity: 0.55; }
</style>
