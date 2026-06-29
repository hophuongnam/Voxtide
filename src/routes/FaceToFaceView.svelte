<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { transcript } from '../lib/stores.svelte';
  import {
    onCoreEvent, startSession, stopSession, getConfig, setConfig, hasApiKey, setApiKey,
    type CoreEvent,
  } from '../lib/ipc';
  import { startMicCapture, stopMicCapture, type MicStats } from '../lib/miccapture';
  import { applyTheme } from '../theme/theme';
  import { LANG_CODES, LANG_NAMES } from '../lib/languages';
  import type { AppConfig } from '../types';

  const ACCOUNT = 'default';
  let cfg = $state<AppConfig | null>(null);
  let hasKey = $state(false);
  let keyInput = $state('');
  let recording = $state(false);
  let err = $state<string | null>(null);
  let mic = $state<MicStats | null>(null); // pipeline vitals for the diag readout
  let events = $state(0); // transcript events received this session
  let unlisten: (() => void) | null = null;

  function handle(ev: CoreEvent) {
    switch (ev.kind) {
      case 'session-started': recording = true; break;
      case 'session-stopped': recording = false; transcript.clearLive(); break;
      case 'transcript-live':
        events++;
        transcript.live({ status: ev.status, text: ev.text, language: ev.language, chip: ev.chip });
        break;
      case 'transcript-final':
        events++;
        transcript.final({ status: ev.status, text: ev.text, language: ev.language, chip: ev.chip, ts_ms: ev.ts_ms });
        break;
      case 'utterance-break': transcript.utteranceBreak(); break;
      case 'error': err = ev.message; break;
    }
  }

  async function record() {
    err = null;
    if (!cfg) return;
    transcript.reset();
    events = 0;
    mic = null;
    try {
      await startSession({
        mode: 'conversation',
        language_a: cfg.language_a,
        language_b: cfg.language_b,
        device_id: '',
        api_key_account: ACCOUNT,
      });
      try {
        await startMicCapture((s) => (mic = s)); // triggers the WebView mic permission prompt
      } catch (e) {
        stopMicCapture(); // release any partially-acquired stream/context
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

  // Persist immediately — every UI pick survives without a save button. Languages
  // lock at session start, so the selects are disabled while recording.
  async function saveLangs() {
    if (cfg) await setConfig(cfg);
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
      applyTheme(cfg.theme); // load --vt-* tokens; without a theme class the view washes out
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
  {:else if cfg}
    <div class="langs">
      <select bind:value={cfg.language_a} onchange={saveLangs} disabled={recording}>
        {#each LANG_CODES as c}<option value={c}>{LANG_NAMES[c]}</option>{/each}
      </select>
      <span>⇄</span>
      <select bind:value={cfg.language_b} onchange={saveLangs} disabled={recording}>
        {#each LANG_CODES as c}<option value={c}>{LANG_NAMES[c]}</option>{/each}
      </select>
    </div>
    <button class="rec" onclick={() => (recording ? stop() : record())}>
      {recording ? '■ Stop' : '● Record'}
    </button>
  {/if}

  {#if err}<p class="err">{err}</p>{/if}
  {#if recording || mic}
    <p class="diag">ctx {mic?.state ?? '—'} · {mic?.sampleRate ?? 0}Hz · sent {mic?.batches ?? 0} · rx {events}</p>
  {/if}

  <section class="lines">
    {#each transcript.original as l}
      <p><b>{l.language ?? '?'}:</b> {l.text}</p>
    {/each}
    {#if transcript.liveOriginal}<p class="live">{transcript.liveOriginal}</p>{/if}
    {#each transcript.translation as l}
      <p class="tr"><b>{l.language ?? '?'}:</b> {l.text}</p>
    {/each}
    {#if transcript.liveTranslation}<p class="live tr">{transcript.liveTranslation}</p>{/if}
  </section>
</main>

<style>
  /* Opaque themed surface — body is transparent, so without this the view
     inherits the WebView's white background and washes out. */
  .ff {
    min-height: 100dvh;
    padding: 16px;
    padding-top: calc(env(safe-area-inset-top, 0px) + 16px);
    background: var(--vt-bg);
    color: var(--vt-text);
    font-family: system-ui, sans-serif;
  }
  header { font-weight: 600; margin-bottom: 12px; }
  .setup { display: flex; gap: 8px; margin-bottom: 12px; }
  .setup input { flex: 1; padding: 10px; }
  .langs { display: flex; align-items: center; gap: 10px; margin-bottom: 12px; }
  .langs select {
    flex: 1; padding: 10px; font-size: 16px; border-radius: 8px;
    background: var(--vt-surface); color: var(--vt-text); border: 1px solid var(--vt-border);
  }
  .langs select:disabled { opacity: 0.5; }
  .rec {
    font-size: 20px; padding: 14px 28px; border-radius: 12px; margin-bottom: 12px;
    background: var(--vt-accent); color: var(--vt-accent-ink); border: none;
  }
  .err { color: var(--vt-danger); }
  .diag { font: 12px ui-monospace, monospace; color: var(--vt-muted); margin: 4px 0 12px; }
  .lines p { margin: 4px 0; }
  .tr { color: var(--vt-accent); }
  .live { opacity: 0.55; }
</style>
