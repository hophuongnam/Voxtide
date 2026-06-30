<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { transcript, splitByLanguage, normLang, coalesceTokens } from '../lib/stores.svelte';
  import {
    onCoreEvent, startSession, stopSession, getConfig, setConfig, hasApiKey, setApiKey, clearApiKey,
    listSessions, getSession, deleteSession,
    type CoreEvent,
  } from '../lib/ipc';
  import { startMicCapture, stopMicCapture, setMicGain, setMicAgc, type MicStats } from '../lib/miccapture';
  import { applyTheme } from '../theme/theme';
  import { LANG_CODES, LANG_NAMES } from '../lib/languages';
  import { formatTime, formatDuration } from '../lib/format';
  import FacePane from '../components/FacePane.svelte';
  import AndroidUpdateBanner from '../components/AndroidUpdateBanner.svelte';
  import type { AppConfig, ConnectionState, SessionRow, TranscriptLine } from '../types';

  const ACCOUNT = 'default';
  let cfg = $state<AppConfig | null>(null);
  let hasKey = $state(false);
  let keyInput = $state('');
  let recording = $state(false);
  let starting = $state(false);
  let err = $state<string | null>(null);
  let mic = $state<MicStats | null>(null); // pipeline vitals for the diag readout
  let events = $state(0); // transcript events received this session
  let conn = $state<ConnectionState>({ state: 'idle', attempt: null, retry_in_ms: null });
  let latency = $state<number | null>(null);
  let unlisten: (() => void) | null = null;
  let removeLifecycleStop: (() => void) | null = null;

  // History: `mode` switches the whole view between live capture, the saved-
  // session list, and read-only replay of a chosen past session.
  let mode = $state<'live' | 'list' | 'replay' | 'settings'>('live');
  let sessions = $state<SessionRow[]>([]);
  let pastOriginal = $state<TranscriptLine[]>([]);
  let pastTranslation = $state<TranscriptLine[]>([]);
  let viewing = $state<SessionRow | null>(null);
  // Replay buckets by the VIEWED session's language_a (which may differ from the
  // current config), so an old en↔vi session splits correctly even if you've
  // since switched to zh↔vi.
  const pastSplit = $derived(viewing
    ? splitByLanguage([...pastOriginal, ...pastTranslation], viewing.lang_a)
    : { far: [] as TranscriptLine[], near: [] as TranscriptLine[] });

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
      case 'session-started': recording = true; conn = { state: 'active', attempt: null, retry_in_ms: null }; break;
      case 'session-stopped': recording = false; conn = { state: 'idle', attempt: null, retry_in_ms: null }; transcript.clearLive(); break;
      case 'transcript-live':
        events++;
        transcript.live({ status: ev.status, text: ev.text, language: ev.language, chip: ev.chip });
        break;
      case 'transcript-final':
        events++;
        transcript.final({ status: ev.status, text: ev.text, language: ev.language, chip: ev.chip, ts_ms: ev.ts_ms });
        break;
      case 'utterance-break': transcript.utteranceBreak(); break;
      case 'connection-state': conn = { state: ev.state, attempt: ev.attempt, retry_in_ms: ev.retry_in_ms }; break;
      case 'latency': latency = ev.median_ms; break;
      case 'error': err = ev.message; break;
    }
  }

  async function record() {
    err = null;
    if (!cfg || starting || recording) return;
    starting = true;
    transcript.reset();
    events = 0;
    mic = null;
    latency = null;
    conn = { state: 'idle', attempt: null, retry_in_ms: null };
    try {
      await startMicCapture((s) => (mic = s), cfg.mic_gain, cfg.mic_agc); // triggers the WebView mic permission prompt
      await startSession({
        mode: 'conversation',
        language_a: cfg.language_a,
        language_b: cfg.language_b,
        device_id: '',
        api_key_account: ACCOUNT,
      });
    } catch (e) {
      stopMicCapture(); // release any partially-acquired stream/context
      try { await stopSession(); } catch {}
      const msg = e instanceof Error ? `${e.name} ${e.message}` : String(e);
      err = msg.startsWith('Error ') ? msg.slice(6) : msg;
    } finally {
      starting = false;
    }
  }

  async function stop() {
    if (!recording && !mic && !starting) return;
    stopMicCapture();
    mic = null;
    starting = false;
    conn = { state: 'idle', attempt: null, retry_in_ms: null };
    try {
      await stopSession();
    } catch (e) {
      err = String(e instanceof Error ? e.message : e);
    }
  }

  function stopFromLifecycle() {
    void stop();
  }

  function stopWhenHidden() {
    if (document.visibilityState === 'hidden') stopFromLifecycle();
  }

  // Persist immediately — every UI pick survives without a save button (langs,
  // swap, mic gain). Languages lock at session start so their selects/swap are
  // disabled while recording; the gain slider stays live (GainNode is mutable).
  async function persistCfg() {
    if (cfg) await setConfig(cfg);
  }

  // AGC toggle: apply live to the active track (best-effort) + persist; takes
  // effect on next record if the WebView ignores the live change.
  function toggleAgc() {
    if (!cfg) return;
    setMicAgc(cfg.mic_agc);
    persistCfg();
  }

  async function swap() {
    if (!cfg || recording || starting) return;
    [cfg.language_a, cfg.language_b] = [cfg.language_b, cfg.language_a];
    await setConfig(cfg);
  }

  async function openHistory() {
    sessions = await listSessions();
    mode = 'list';
  }
  async function openSession(row: SessionRow) {
    const { tokens } = await getSession(row.id);
    const c = coalesceTokens(tokens);
    pastOriginal = c.original;
    pastTranslation = c.translation;
    viewing = row;
    mode = 'replay';
  }
  function backToList() { mode = 'list'; viewing = null; }
  function openSettings() { mode = 'settings'; }
  function exitToLive() { mode = 'live'; viewing = null; pastOriginal = []; pastTranslation = []; }
  async function removeSession(row: SessionRow) {
    const langs = `${row.lang_a.toUpperCase()}→${row.lang_b.toUpperCase()}`;
    if (!confirm(`Delete the ${langs} session from ${formatTime(row.started_at)}?`)) return;
    await deleteSession(row.id);
    sessions = await listSessions();
  }

  async function saveKey() {
    if (!keyInput.trim()) return;
    await setApiKey(ACCOUNT, keyInput.trim());
    keyInput = '';
    hasKey = true;
  }
  async function clearKey() {
    await clearApiKey(ACCOUNT);
    hasKey = false;
  }

  onMount(async () => {
    if (typeof window !== 'undefined') {
      window.addEventListener('voxtide:android-stop', stopFromLifecycle);
      document.addEventListener('visibilitychange', stopWhenHidden);
      removeLifecycleStop = () => {
        window.removeEventListener('voxtide:android-stop', stopFromLifecycle);
        document.removeEventListener('visibilitychange', stopWhenHidden);
      };
    }
    // Guard Tauri calls so vitest/jsdom doesn't choke.
    if (typeof window !== 'undefined' && (window as any).__TAURI_INTERNALS__) {
      cfg = await getConfig();
      applyTheme(cfg.theme); // load --vt-* tokens; without a theme class the view washes out
      hasKey = await hasApiKey(ACCOUNT);
      unlisten = await onCoreEvent(handle);
    }
  });
  onDestroy(() => {
    removeLifecycleStop?.();
    unlisten?.();
    stopMicCapture();
  });
</script>

<main class="ff">
  {#if !hasKey}
    <div class="setup">
      <input type="password" placeholder="Soniox API key" bind:value={keyInput} />
      <button onclick={saveKey}>Save key</button>
    </div>
  {:else if mode === 'list'}
    <header class="hh">
      <span>History</span>
      <button class="hclose" onclick={exitToLive} aria-label="Close history">✕</button>
    </header>
    <div class="hist">
      {#if sessions.length === 0}
        <p class="empty">No saved sessions yet.</p>
      {:else}
        {#each sessions as s}
          <div class="srow">
            <button class="srow-main" onclick={() => openSession(s)}>
              <span class="stime">{formatTime(s.started_at)}</span>
              <span class="slangs">{s.lang_a.toUpperCase()} → {s.lang_b.toUpperCase()}</span>
              <span class="sdur">{s.duration_ms ? formatDuration(s.duration_ms) : '—'}</span>
            </button>
            <button class="srow-del" onclick={() => removeSession(s)} aria-label="Delete session">🗑</button>
          </div>
        {/each}
      {/if}
    </div>
  {:else if mode === 'settings' && cfg}
    <header class="hh">
      <span>Settings</span>
      <button class="hclose" onclick={exitToLive} aria-label="Close settings">✕</button>
    </header>
    <div class="settings">
      <section class="sset">
        <div class="slabel">Soniox API key</div>
        <input class="sinput" type="password" autocomplete="off"
               placeholder={hasKey ? '•••••••• (saved)' : 'sk_…'}
               bind:value={keyInput} aria-label="Soniox API key" />
        <div class="sbtns">
          <button class="sbtn-primary" onclick={saveKey}>Save</button>
          {#if hasKey}<button class="sbtn" onclick={clearKey}>Remove</button>{/if}
        </div>
      </section>
      <section class="sset">
        <label class="agc-set">
          <input type="checkbox" bind:checked={cfg.mic_agc} onchange={toggleAgc} />
          Auto gain control
        </label>
        <p class="sdesc">Lets the mic automatically adjust loudness. Off by default — leave it off to set the level yourself with the gain slider.</p>
      </section>
    </div>
  {:else if mode === 'replay' && viewing}
    <FacePane lines={pastSplit.far} live={[]} follow={false} />
    <div class="bar">
      <div class="ctl">
        <button class="swap" onclick={backToList} aria-label="Back to sessions">‹ Sessions</button>
        <span class="rinfo">{viewing.lang_a.toUpperCase()} → {viewing.lang_b.toUpperCase()} · {formatTime(viewing.started_at)}</span>
      </div>
    </div>
    <FacePane lines={pastSplit.near} live={[]} follow={false} />
  {:else if cfg}
    <FacePane lines={split.far} live={live.far} rotated />

    <div class="bar">
      <AndroidUpdateBanner />
      <div class="ctl">
        <select bind:value={cfg.language_a} onchange={persistCfg} disabled={recording || starting}>
          {#each LANG_CODES as c}<option value={c}>{LANG_NAMES[c]}</option>{/each}
        </select>
        <button class="swap" onclick={swap} disabled={recording || starting} aria-label="Swap languages">⇄</button>
        <select bind:value={cfg.language_b} onchange={persistCfg} disabled={recording || starting}>
          {#each LANG_CODES as c}<option value={c}>{LANG_NAMES[c]}</option>{/each}
        </select>
        <button class="rec" class:on={recording} disabled={starting} onclick={() => (recording ? stop() : record())}
                aria-label={recording ? 'Stop' : 'Record'}>{starting ? '...' : recording ? '■' : '●'}</button>
      </div>
      <div class="gain">
        <button class="hist-btn" onclick={openHistory} disabled={recording || starting} aria-label="History">🕘</button>
        <button class="hist-btn" onclick={openSettings} disabled={recording || starting} aria-label="Settings">⚙️</button>
        <span class="gl" aria-hidden="true">🎤</span>
        <input class="gslider" type="range" min="0.5" max="4" step="0.1"
               bind:value={cfg.mic_gain}
               oninput={() => cfg && setMicGain(cfg.mic_gain)} onchange={persistCfg}
               aria-label="Mic sensitivity" />
        <span class="gv">{cfg.mic_gain.toFixed(1)}×</span>
      </div>
      {#if err}<p class="err">{err}</p>{/if}
      {#if recording || starting || mic}
        <p class="diag">ctx {mic?.state ?? '—'} · {mic?.sampleRate ?? 0}Hz · sent {mic?.batches ?? 0} · rx {events} · {conn.state}{latency ? ` · ${latency}ms` : ''}</p>
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
  .gain { display: flex; align-items: center; gap: 8px; margin-top: 8px; }
  .gl { font-size: 15px; }
  .gslider { flex: 1; min-width: 0; accent-color: var(--vt-accent); }
  .gv { font: 12px ui-monospace, monospace; color: var(--vt-muted); min-width: 2.6em; text-align: right; }
  /* Settings screen */
  .settings { flex: 1; min-height: 0; overflow-y: auto; padding: 16px; }
  .sset { padding-bottom: 16px; margin-bottom: 16px; border-bottom: 1px solid var(--vt-border); }
  .slabel { font-size: 13px; font-weight: 600; margin-bottom: 8px; }
  .sinput {
    width: 100%; padding: 10px; font-size: 15px; border-radius: 8px; margin-bottom: 10px;
    background: var(--vt-surface); color: var(--vt-text); border: 1px solid var(--vt-border);
  }
  .sbtns { display: flex; gap: 8px; }
  .sbtn-primary {
    padding: 8px 16px; border-radius: 8px; border: none; font-weight: 600;
    background: var(--vt-accent); color: var(--vt-accent-ink);
  }
  .sbtn {
    padding: 8px 16px; border-radius: 8px;
    background: transparent; color: var(--vt-muted); border: 1px solid var(--vt-border);
  }
  .agc-set { display: flex; align-items: center; gap: 8px; font-size: 15px; color: var(--vt-text); }
  .agc-set input { accent-color: var(--vt-accent); width: 18px; height: 18px; }
  .sdesc { margin: 8px 0 0; font-size: 13px; line-height: 1.4; color: var(--vt-muted); }
  .err { color: var(--vt-danger); margin: 6px 0 0; font-size: 13px; }
  .diag { font: 11px ui-monospace, monospace; color: var(--vt-muted); margin: 6px 0 0; }

  .hist-btn {
    padding: 8px 10px; font-size: 16px; border-radius: 8px; flex: none;
    background: var(--vt-bg); color: var(--vt-text); border: 1px solid var(--vt-border);
  }
  .hist-btn:disabled { opacity: 0.5; }
  /* History list + replay */
  .hh {
    display: flex; align-items: center; justify-content: space-between;
    padding: 14px 16px; border-bottom: 1px solid var(--vt-border); font-weight: 600;
  }
  .hclose { background: none; border: none; color: var(--vt-text); font-size: 20px; }
  .hist { flex: 1; min-height: 0; overflow-y: auto; padding: 8px 12px; }
  .empty { color: var(--vt-muted); text-align: center; padding: 48px 0; }
  .srow { display: flex; align-items: stretch; gap: 6px; margin-bottom: 6px; }
  .srow-main {
    flex: 1; min-width: 0; display: flex; align-items: center; gap: 10px;
    padding: 12px; border-radius: 8px; text-align: left;
    background: var(--vt-surface); color: var(--vt-text); border: 1px solid var(--vt-border);
  }
  .srow-del {
    flex: none; padding: 0 14px; border-radius: 8px; font-size: 16px;
    background: var(--vt-surface); color: var(--vt-muted); border: 1px solid var(--vt-border);
  }
  .stime { font-size: 15px; }
  .slangs { font: 12px ui-monospace, monospace; color: var(--vt-muted); }
  .sdur { margin-left: auto; font: 12px ui-monospace, monospace; color: var(--vt-subtle); }
  .rinfo { font: 12px ui-monospace, monospace; color: var(--vt-muted); margin-left: 8px; }
</style>
