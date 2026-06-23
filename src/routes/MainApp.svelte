<script lang="ts">
  import { onMount } from 'svelte';
  import VoxWindow from '../components/chrome/VoxWindow.svelte';
  import Toolbar from '../components/toolbar/Toolbar.svelte';
  import Sidebar from '../components/sidebar/Sidebar.svelte';
  import TranscriptPane from '../components/transcript/TranscriptPane.svelte';
  import EmptyState from '../components/transcript/EmptyState.svelte';
  import NoApiKey from '../components/transcript/NoApiKey.svelte';
  import StatusBar from '../components/status/StatusBar.svelte';
  import SettingsSheet from '../components/settings/SettingsSheet.svelte';

  import PermissionBanner from '../components/PermissionBanner.svelte';
  import UpdateBanner from '../components/UpdateBanner.svelte';
  import { applyTheme } from '../theme/theme';
  import { listen } from '@tauri-apps/api/event';
  import { check, type Update } from '@tauri-apps/plugin-updater';
  import { relaunch } from '@tauri-apps/plugin-process';
  import {
    appInfo as fetchAppInfo, deleteSession,
    getConfig, getSession, hasApiKey, listLoopbackSources, listMics, listSessions,
    onCoreEvent, onOverlayVisibility, searchTranscripts, startSession, stopSession,
    showOverlay, hideOverlay,
  } from '../lib/ipc';
  import ConfirmDeleteSheet from '../components/sidebar/ConfirmDeleteSheet.svelte';
  import { coalesceTokens, transcript, session, config, devices } from '../lib/stores.svelte';
  import { langByCode } from '../lib/languages';
  import type { TranscriptLine } from '../types';
  import type { AppInfo, CoreEvent, DeviceEntry } from '../lib/ipc';
  import type { AppConfig, FontSize, Mode, SessionRow, StartError } from '../types';

  let mode = $state<Mode>('meeting');
  // System Audio only: blend the local mic in (→ two-way). Hydrated from config.
  let captureMic = $state(false);
  let sessions = $state<SessionRow[]>([]);
  let query = $state('');
  let searchHits = $state<SessionRow[]>([]);
  let settingsOpen = $state(false);
  let overlayShown = $state(false);
  let elapsedMs = $state(0);
  let mainWidth = $state(920);
  // Backend-reported model/format facts; null until the boot fetch lands
  // (the status bar shows an em-dash placeholder, never a stale literal).
  let backendInfo = $state<AppInfo | null>(null);
  let selectedSource = $state<DeviceEntry | null>(null);
  let permissionKind = $state<'mic' | 'audio-capture' | null>(null);
  // Plain, dismissible error strip for failures that aren't a permission prompt:
  // a structured start error (device-missing/other), a non-structured rejection,
  // or a provider `error` core event (e.g. a rejected Soniox API key).
  let appError = $state<string | null>(null);

  // Past-session viewer. null = follow live capture; otherwise show pastOriginal/pastTranslation.
  let viewingId = $state<number | null>(null);
  // The stored row of the session being viewed: its OWN mode/languages label
  // the pane (the current config may have changed since it was recorded).
  let viewingSession = $state<SessionRow | null>(null);
  let pastOriginal = $state<TranscriptLine[]>([]);
  let pastTranslation = $state<TranscriptLine[]>([]);

  // Delete-transcript flow.
  let pendingDelete = $state<SessionRow | null>(null);
  let deleting = $state(false);
  let deleteError = $state<string | null>(null);

  // Auto-update flow. Silent check on launch; banner appears only if an update is found.
  let pendingUpdate = $state<Update | null>(null);
  let updateInstalling = $state(false);
  let updateProgress = $state<number | null>(null);
  let updateError = $state<string | null>(null);
  let updateDismissed = $state(false);

  async function onUpdateInstall() {
    if (!pendingUpdate) return;
    updateInstalling = true;
    updateError = null;
    updateProgress = 0;
    try {
      let downloaded = 0;
      let contentLength = 0;
      await pendingUpdate.downloadAndInstall((event) => {
        if (event.event === 'Started') {
          contentLength = event.data.contentLength ?? 0;
        } else if (event.event === 'Progress') {
          downloaded += event.data.chunkLength;
          updateProgress = contentLength > 0
            ? Math.min(100, Math.round((downloaded / contentLength) * 100))
            : null;
        }
      });
      await relaunch();
    } catch (e) {
      updateError = String(e instanceof Error ? e.message : e);
      updateInstalling = false;
    }
  }
  function onUpdateDismiss() {
    pendingUpdate = null;
    updateError = null;
    updateDismissed = true;
  }

  function onDeleteRequest(row: SessionRow) {
    pendingDelete = row;
    deleteError = null;
  }
  function onDeleteCancel() {
    pendingDelete = null;
    deleteError = null;
  }
  async function onDeleteConfirm() {
    const target = pendingDelete;
    if (!target) return;
    deleting = true;
    deleteError = null;
    try {
      await deleteSession(target.id);
      // Drop from sidebar list immediately and re-fetch authoritative state.
      sessions = sessions.filter(s => s.id !== target.id);
      searchHits = searchHits.filter(s => s.id !== target.id);
      if (viewingId === target.id) {
        viewingId = null;
        viewingSession = null;
        pastOriginal = [];
        pastTranslation = [];
      }
      sessions = await listSessions();
      pendingDelete = null;
    } catch (e) {
      deleteError = String(e instanceof Error ? e.message : e);
    } finally {
      deleting = false;
    }
  }

  async function onSelectSession(id: number) {
    // Clicking the currently-recording session returns to the live view.
    if (session.recording && id === session.sessionId) {
      onReturnToLive();
      return;
    }
    try {
      const { session: row, tokens } = await getSession(id);
      const { original, translation } = coalesceTokens(tokens);
      pastOriginal = original;
      pastTranslation = translation;
      viewingSession = row;
      viewingId = id;
    } catch (e) {
      console.error('getSession failed', e);
    }
  }

  function onReturnToLive() {
    viewingId = null; viewingSession = null;
    pastOriginal = []; pastTranslation = [];
  }

  const langA = $derived(langByCode(config.config?.language_a ?? 'en'));
  const langB = $derived(langByCode(config.config?.language_b ?? 'vi'));
  const fontSize: FontSize = $derived(config.config?.font_size ?? 'm');
  const showPinyin: boolean = $derived(config.config?.show_pinyin ?? false);

  const meetingSources = $derived(devices.loopbacks);
  const micSources     = $derived(devices.mics);

  $effect(() => {
    const list = mode === 'meeting' ? devices.loopbacks : devices.mics;
    if (list.length === 0) return;
    if (selectedSource && !list.some(s => s.id === selectedSource!.id)) {
      selectedSource = null;
    }
    if (selectedSource) return;
    const savedId = mode === 'meeting'
      ? config.config?.default_meeting_source
      : config.config?.default_mic;
    const saved = savedId ? list.find(s => s.id === savedId) : null;
    selectedSource = saved ?? list[0]!;
  });

  function refreshSources() {
    listLoopbackSources().then(v => devices.setLoopbacks(v)).catch(() => {});
    listMics().then(v => devices.setMics(v)).catch(() => {});
  }

  function handleCoreEvent(ev: CoreEvent) {
    switch (ev.kind) {
      case 'session-started':
        session.start(ev.session_id, Date.now());
        // Snap to live view when a new capture starts so the user sees what they just initiated.
        viewingId = null; viewingSession = null; pastOriginal = []; pastTranslation = [];
        // Refetch so the new row (the live one) appears in the sidebar now,
        // not only after the session ends.
        listSessions().then(v => sessions = v);
        break;
      case 'session-stopped':
        // Stale guard: a worker that outlived stop()'s join can emit its
        // SessionStopped AFTER a newer session started — ignore any stop
        // that isn't for the session we're tracking.
        if (session.sessionId !== null && ev.session_id !== session.sessionId) break;
        session.stop();
        // Drop the in-flight partial: nothing will finalize it now, and a
        // leftover live line blinks forever under the committed transcript.
        transcript.clearLive();
        listSessions().then(v => sessions = v);
        break;
      case 'transcript-live':
        transcript.live({ status: ev.status, text: ev.text, language: ev.language, chip: ev.chip }); break;
      case 'transcript-final':
        transcript.final({ status: ev.status, text: ev.text, language: ev.language, chip: ev.chip, ts_ms: ev.ts_ms }); break;
      case 'utterance-break':
        transcript.utteranceBreak(); break;
      case 'connection-state':
        session.setConnection(ev.state, ev.attempt, ev.retry_in_ms); break;
      case 'latency': session.setLatency(ev.median_ms); break;
      case 'error': appError = ev.message; break;
    }
  }

  // Silent update check. Skipped outside the Tauri runtime (vitest, vite
  // preview). Fire-and-forget so a slow CDN can't delay anything; errors
  // (offline, no manifest yet, bad signature) stay in the console — never
  // block the app or surface a banner unless there's an update.
  // `minIntervalMs` rate-limits opportunistic call sites (window focus).
  let lastUpdateCheck = 0;
  function checkForUpdate(minIntervalMs = 0) {
    if (!(window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__) return;
    const now = Date.now();
    if (now - lastUpdateCheck < minIntervalMs) return;
    lastUpdateCheck = now;
    void check()
      .then((update) => {
        if (update && !updateDismissed) pendingUpdate = update;
      })
      .catch((e) => console.debug('updater check failed', e));
  }

  onMount(() => {
    let unlisten: (() => void) | undefined;
    let unHotkey: (() => void) | undefined;
    let unOverlay: (() => void) | undefined;
    const ro = new ResizeObserver((entries) => {
      const first = entries[0];
      if (first) mainWidth = Math.round(first.contentRect.width);
    });
    ro.observe(document.body);
    const tick = setInterval(() => {
      if (session.recording && session.startedAt) elapsedMs = Date.now() - session.startedAt;
    }, 250);
    // The process can live for weeks under close-to-hide, so one boot-time
    // check is not enough: re-check every 6 h, plus on window focus (the
    // moment the user comes back is when an update prompt is actually seen)
    // rate-limited to hourly.
    const updateTick = setInterval(() => checkForUpdate(), 6 * 60 * 60 * 1000);
    const onFocus = () => checkForUpdate(60 * 60 * 1000);
    window.addEventListener('focus', onFocus);

    (async () => {
      try {
        // Listeners attach FIRST: if any boot fetch below rejects, the app
        // must still react to core events and the hotkey. (The old order put
        // the fallible fetches first with no catch — one rejection left the
        // whole app inert: no event listener, no hotkey, no updater.)
        unlisten = await onCoreEvent(handleCoreEvent);
        unHotkey = await listen('voxtide://hotkey/toggle', async () => {
          if (session.recording) await onStop();
          else await onStart();
        });
        // Track the overlay's REAL visibility (it can hide itself, or be
        // shown/hidden from another window) so the toolbar toggle never
        // drifts out of sync with the actual window state.
        unOverlay = await onOverlayVisibility((visible) => { overlayShown = visible; });

        checkForUpdate();

        // Cosmetic status-bar facts — fire-and-forget so a failure can't
        // block boot or surface as a startup error.
        void fetchAppInfo()
          .then((info) => { backendInfo = info; })
          .catch((e) => console.debug('app_info failed', e));

        const cfg = await getConfig();
        config.setConfig(cfg);
        mode = cfg.mode;
        captureMic = cfg.meeting_capture_mic;
        applyTheme(cfg.theme);
        config.setHasApiKey(await hasApiKey(config.apiKeyAccount));
        sessions = await listSessions();
        refreshSources();
      } catch (e) {
        appError = `startup: ${e instanceof Error ? e.message : e}`;
      }
    })();

    return () => {
      unlisten?.(); unHotkey?.(); unOverlay?.(); ro.disconnect();
      clearInterval(tick); clearInterval(updateTick);
      window.removeEventListener('focus', onFocus);
      clearTimeout(searchTimer);
    };
  });

  // Debounced, staleness-guarded search. The old per-keystroke await had no
  // sequence guard (a slow early response could overwrite a newer one) and
  // mapped hits onto the in-memory `sessions` cache, silently dropping any
  // match in a session older than the sidebar's 50 rows.
  let searchSeq = 0;
  let searchTimer: ReturnType<typeof setTimeout> | undefined;
  function onSearch(q: string) {
    query = q;
    if (!q.trim()) {
      clearTimeout(searchTimer);
      searchHits = [];
      return;
    }
    clearTimeout(searchTimer);
    const seq = ++searchSeq;
    searchTimer = setTimeout(async () => {
      try {
        const rows = await searchTranscripts(q);
        if (seq === searchSeq) searchHits = rows;
      } catch (e) {
        if (seq === searchSeq) appError = `search: ${e instanceof Error ? e.message : e}`;
      }
    }, 200);
  }

  // Type guard for the structured rejection payload `start_session` returns.
  function isStartError(e: unknown): e is StartError {
    return typeof e === 'object' && e !== null
      && typeof (e as { kind?: unknown }).kind === 'string';
  }

  async function onStart() {
    if (!config.hasApiKey || !config.config || !selectedSource) return;
    transcript.reset();
    // Clear both surfaces up front so a retry never stacks a stale banner/strip.
    appError = null;
    permissionKind = null;
    try {
      await startSession({
        mode,
        language_a: config.config.language_a,
        language_b: config.config.language_b,
        device_id: selectedSource.id,
        api_key_account: config.apiKeyAccount,
        capture_mic: mode === 'meeting' && captureMic,
        mic_device_id: config.config.default_mic ?? '',
      });
    } catch (e) {
      // Route the typed StartError; never rethrow (a rethrow = unhandled
      // rejection with no UI). A permission denial opens the banner; everything
      // else (missing device, or any other failure) shows the plain strip. The
      // two surfaces are mutually exclusive — only one is set per failure.
      if (isStartError(e)) {
        switch (e.kind) {
          case 'mic-permission':     permissionKind = 'mic'; break;
          case 'capture-permission': permissionKind = 'audio-capture'; break;
          default:                   appError = e.message; // device-missing | other
        }
      } else {
        // Non-structured rejection (plain string / Error): surface verbatim.
        appError = String(e instanceof Error ? e.message : e);
      }
    }
  }

  async function onStop() {
    try {
      await stopSession();
    } catch (e) {
      appError = String(e instanceof Error ? e.message : e);
    }
  }
  // One guarded persist path for every settings mutation (was copy-pasted
  // five times as `await setConfig(next); config.setConfig(next)`); failures
  // surface in the error strip instead of as unhandled rejections.
  function persist(patch: Partial<AppConfig>): Promise<void> {
    return config.update(patch).catch((e) => {
      appError = `settings: ${(e as { message?: string })?.message ?? e}`;
    });
  }
  async function onModeChange(m: Mode) {
    if (m === mode) return;
    mode = m;
    if (config.config && config.config.mode !== m) await persist({ mode: m });
  }
  async function onCaptureMicChange(v: boolean) {
    captureMic = v;
    if (config.config && config.config.meeting_capture_mic !== v) {
      await persist({ meeting_capture_mic: v });
    }
  }
  async function onSourceChange(d: DeviceEntry) {
    selectedSource = d;
    const c = config.config;
    if (!c) return;
    if (mode === 'meeting') {
      if (c.default_meeting_source !== d.id) await persist({ default_meeting_source: d.id });
    } else if (c.default_mic !== d.id) {
      await persist({ default_mic: d.id });
    }
  }
  async function onSwap()       {
    const c = config.config;
    if (!c) return;
    // Swap source (a) and target (b) languages.
    await persist({ language_a: c.language_b, language_b: c.language_a });
  }
  async function onReadingChange(next: AppConfig) {
    const c = config.config;
    if (!c) return;
    if (c.font_size === next.font_size && c.show_pinyin === next.show_pinyin) return;
    await persist({ font_size: next.font_size, show_pinyin: next.show_pinyin });
  }
  async function onLangPick(which: 'a' | 'b', code: string) {
    const c = config.config;
    if (!c) return;
    if (which === 'a' && code === c.language_b) return;
    if (which === 'b' && code === c.language_a) return;
    await persist(which === 'a' ? { language_a: code } : { language_b: code });
  }
  async function onOverlayToggle() {
    if (overlayShown) { await hideOverlay(); overlayShown = false; }
    else              { await showOverlay(); overlayShown = true;  }
  }
  function onSettings() { settingsOpen = true; }

  const summary = $derived(mode === 'meeting'
    ? `one_way → ${langB.code}` : `two_way · ${langA.code} ⇄ ${langB.code}`);
  // "s16le" is the one display literal kept frontend-side (it names the wire
  // format, pcm_s16le); rate and channel count come from the backend.
  const audioFormat = $derived(backendInfo
    ? `${backendInfo.sample_rate_hz / 1000} kHz · ${backendInfo.channels === 1 ? 'mono' : `${backendInfo.channels} ch`} · s16le`
    : '—');
</script>

<VoxWindow>
  <Toolbar
    {mode} onmode={onModeChange}
    recording={session.recording}
    onstart={onStart} onstop={onStop}
    onsettings={onSettings} onoverlay={onOverlayToggle}
    overlayShown={overlayShown}
    a={langA} b={langB}
    onswap={onSwap}
    onlangpick={onLangPick}
    source={selectedSource}
    sourceOptions={mode === 'meeting' ? meetingSources : micSources}
    onsource={onSourceChange}
    {captureMic} oncapturemic={onCaptureMicChange} />

  <PermissionBanner kind={permissionKind} ondismiss={() => permissionKind = null} />
  {#if appError}
    <div data-testid="app-error" class="px-4 py-2.5 flex items-center gap-3"
         style:background="var(--vt-danger-tint)"
         style:border-bottom="0.5px solid var(--vt-danger-border)">
      <span class="block w-2 h-2 rounded-full shrink-0" style:background="var(--vt-danger)"></span>
      <div class="flex-1 text-[11.5px] leading-snug" style:color="var(--vt-text)">{appError}</div>
      <button type="button" onclick={() => appError = null} aria-label="Dismiss error"
              class="bg-transparent border-0 cursor-pointer px-2 py-1 text-[13px] leading-none"
              style:color="var(--vt-muted)">✕</button>
    </div>
  {/if}
  <UpdateBanner
    version={pendingUpdate?.version ?? null}
    busy={updateInstalling}
    progress={updateProgress}
    error={updateError}
    oninstall={onUpdateInstall}
    ondismiss={onUpdateDismiss} />

  <div class="flex-1 flex overflow-hidden">
    <Sidebar
      sessions={query.trim() ? searchHits : sessions}
      activeId={viewingId ?? session.sessionId}
      liveId={session.recording ? session.sessionId : null}
      onselect={onSelectSession}
      onsearch={onSearch}
      query={query}
      ondeleterequest={onDeleteRequest} />

    <div class="flex-1 flex flex-col min-w-0">
      {#if !config.hasApiKey}
        <NoApiKey onaddkey={() => settingsOpen = true} />
      {:else if viewingId !== null}
        {#if session.recording}
          <button type="button" onclick={onReturnToLive}
                  class="px-4 py-2 flex items-center gap-3 cursor-pointer w-full text-left border-0"
                  style:background="var(--vt-accent-tint-10)"
                  style:border-bottom="0.5px solid var(--vt-accent-tint-25)">
            <span class="block w-2 h-2 rounded-full"
                  style:background="var(--vt-rec)"
                  style:box-shadow="0 0 0 3px var(--vt-rec-glow)"></span>
            <span class="text-[12px] font-semibold" style:color="var(--vt-text)">Recording in progress</span>
            <span class="ml-auto text-[12px] font-semibold" style:color="var(--vt-accent)">Return to live →</span>
          </button>
        {/if}
        <TranscriptPane
          mode={(viewingSession?.mode as Mode) ?? mode}
          a={viewingSession ? langByCode(viewingSession.lang_a) : langA}
          b={viewingSession ? langByCode(viewingSession.lang_b) : langB}
          original={pastOriginal}
          translation={pastTranslation}
          liveOriginal=""
          liveTranslation=""
          {fontSize} {showPinyin}
          cfg={config.config} onconfigchange={onReadingChange} />
      {:else if !session.recording && transcript.original.length === 0 && transcript.translation.length === 0}
        <EmptyState {mode} />
      {:else}
        <TranscriptPane
          {mode} a={langA} b={langB}
          original={transcript.original}
          translation={transcript.translation}
          liveOriginal={transcript.liveOriginal}
          liveTranslation={transcript.liveTranslation}
          liveOriginalLang={transcript.liveOriginalLang}
          liveTranslationLang={transcript.liveTranslationLang}
          {fontSize} {showPinyin}
          cfg={config.config} onconfigchange={onReadingChange} />
      {/if}
      <StatusBar
        recording={session.recording}
        elapsedMs={elapsedMs}
        latencyMs={session.latencyMs}
        {mode}
        translationSummary={summary}
        model={backendInfo?.model ?? '—'}
        audioFormat={audioFormat}
        version={__APP_VERSION__}
        width={mainWidth - 240} />
    </div>
  </div>
</VoxWindow>

<SettingsSheet open={settingsOpen} onclose={() => settingsOpen = false} />
<ConfirmDeleteSheet
  open={pendingDelete !== null}
  target={pendingDelete}
  busy={deleting}
  error={deleteError}
  onconfirm={onDeleteConfirm}
  oncancel={onDeleteCancel} />
