<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { transcript, splitByLanguage, normLang } from '../lib/stores.svelte';
  import {
    onCoreEvent, startSession, stopSession, getConfig, setConfig, hasApiKey, setApiKey,
    type CoreEvent,
  } from '../lib/ipc';
  import { startMicCapture, stopMicCapture, type MicStats } from '../lib/miccapture';
  import { applyTheme } from '../theme/theme';
  import { LANG_CODES, LANG_NAMES } from '../lib/languages';
  import FacePane from '../components/FacePane.svelte';
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

  // Far pane = language_a (the person across the table, rotated 180°); near pane
  // = everything else (you). splitByLanguage merges original+translation so each
  // reader gets one monolingual stream; the catch-all near pane means an
  // unexpected Soniox tag can never make a line vanish.
  const aNorm = $derived(normLang(cfg?.language_a));
  const split = $derived(
    splitByLanguage([...transcript.original, ...transcript.translation], cfg?.language_a ?? ''),
  );
  // Route in-flight partials to a pane by detected language, falling back to the
  // column's configured code so an unresolved live lang doesn't flicker through
  // the catch-all near pane before it finalizes.
  const live = $derived.by(() => {
    const far: { text: string; translated: boolean }[] = [];
    const near: { text: string; translated: boolean }[] = [];
    const route = (text: string, lang: string | null, fallback: string, translated: boolean) => {
      if (!text) return;
      (normLang(lang ?? fallback) === aNorm ? far : near).push({ text, translated });
    };
    route(transcript.liveOriginal, transcript.liveOriginalLang, cfg?.language_a ?? '', false);
    route(transcript.liveTranslation, transcript.liveTranslationLang, cfg?.language_b ?? '', true);
    return { far, near };
  });

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
  // lock at session start, so the selects/swap are disabled while recording.
  async function saveLangs() {
    if (cfg) await setConfig(cfg);
  }

  async function swap() {
    if (!cfg || recording) return;
    [cfg.language_a, cfg.language_b] = [cfg.language_b, cfg.language_a];
    await setConfig(cfg);
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
  {#if !hasKey}
    <div class="setup">
      <input type="password" placeholder="Soniox API key" bind:value={keyInput} />
      <button onclick={saveKey}>Save key</button>
    </div>
  {:else if cfg}
    <FacePane lines={split.far} live={live.far} rotated />

    <div class="bar">
      <div class="ctl">
        <select bind:value={cfg.language_a} onchange={saveLangs} disabled={recording}>
          {#each LANG_CODES as c}<option value={c}>{LANG_NAMES[c]}</option>{/each}
        </select>
        <button class="swap" onclick={swap} disabled={recording} aria-label="Swap languages">⇄</button>
        <select bind:value={cfg.language_b} onchange={saveLangs} disabled={recording}>
          {#each LANG_CODES as c}<option value={c}>{LANG_NAMES[c]}</option>{/each}
        </select>
        <button class="rec" class:on={recording} onclick={() => (recording ? stop() : record())}
                aria-label={recording ? 'Stop' : 'Record'}>{recording ? '■' : '●'}</button>
      </div>
      {#if err}<p class="err">{err}</p>{/if}
      {#if recording || mic}
        <p class="diag">ctx {mic?.state ?? '—'} · {mic?.sampleRate ?? 0}Hz · sent {mic?.batches ?? 0} · rx {events}</p>
      {/if}
    </div>

    <FacePane lines={split.near} live={live.near} />
  {/if}
</main>

<style>
  /* Full-height themed surface split top/bottom. body is transparent, so the
     opaque bg here is what stops the Android WebView washing out to white.
     Insets keep the panes clear of the status/nav bars; the controls live in
     the center divider, which is never under a system bar (fixes the old
     un-tappable-toolbar bug). */
  .ff {
    display: flex;
    flex-direction: column;
    height: 100dvh;
    padding-top: env(safe-area-inset-top, 0px);
    padding-bottom: env(safe-area-inset-bottom, 0px);
    background: var(--vt-bg);
    color: var(--vt-text);
    font-family: system-ui, sans-serif;
  }
  .setup { display: flex; gap: 8px; padding: 24px 16px; }
  .setup input { flex: 1; padding: 10px; }

  .bar {
    border-top: 1px solid var(--vt-border);
    border-bottom: 1px solid var(--vt-border);
    background: var(--vt-surface);
    padding: 8px 12px;
  }
  .ctl { display: flex; align-items: center; gap: 8px; }
  .ctl select {
    flex: 1; min-width: 0; padding: 9px; font-size: 15px; border-radius: 8px;
    background: var(--vt-bg); color: var(--vt-text); border: 1px solid var(--vt-border);
  }
  .ctl select:disabled, .swap:disabled { opacity: 0.5; }
  .swap {
    padding: 8px 10px; font-size: 16px; border-radius: 8px;
    background: var(--vt-bg); color: var(--vt-text); border: 1px solid var(--vt-border);
  }
  .rec {
    width: 48px; height: 48px; flex: none; font-size: 18px; border-radius: 50%;
    background: var(--vt-accent); color: var(--vt-accent-ink); border: none;
  }
  .rec.on { background: var(--vt-rec); }
  .err { color: var(--vt-danger); margin: 6px 0 0; font-size: 13px; }
  .diag { font: 11px ui-monospace, monospace; color: var(--vt-muted); margin: 6px 0 0; }
</style>
