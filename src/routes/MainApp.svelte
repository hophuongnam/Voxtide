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
  import { applyTheme } from '../theme/theme';
  import {
    getConfig, hasApiKey, listLoopbackSources, listMics, listSessions,
    onCoreEvent, searchTranscripts, setConfig, startSession, stopSession,
    showOverlay, hideOverlay,
  } from '../lib/ipc';
  import { transcript, session, config, devices } from '../lib/stores.svelte';
  import type { CoreEvent, DeviceEntry } from '../lib/ipc';
  import type { Mode, SessionRow, WhichLang } from '../types';

  // Language lookup table — spec keeps it small.
  const LANG_NAMES: Record<string, string> = {
    en: 'English', vi: 'Vietnamese', ja: 'Japanese', ko: 'Korean',
    es: 'Spanish', de: 'German', fr: 'French', zh: 'Chinese',
  };

  let mode = $state<Mode>('meeting');
  let sessions = $state<SessionRow[]>([]);
  let query = $state('');
  let searchHits = $state<SessionRow[]>([]);
  let settingsOpen = $state(false);
  let overlayShown = $state(false);
  let elapsedMs = $state(0);
  let mainWidth = $state(920);
  let selectedSource = $state<DeviceEntry | null>(null);
  let permissionKind = $state<'mic' | 'audio-capture' | null>(null);

  const langA = $derived({ code: (config.config?.language_a ?? 'en').toUpperCase(),
                           name: LANG_NAMES[config.config?.language_a ?? 'en'] ?? '' });
  const langB = $derived({ code: (config.config?.language_b ?? 'vi').toUpperCase(),
                           name: LANG_NAMES[config.config?.language_b ?? 'vi'] ?? '' });
  const mine: WhichLang = $derived(config.config?.mine ?? 'b');

  const meetingSources = $derived(devices.loopbacks);
  const micSources     = $derived(devices.mics);

  function refreshSources() {
    listLoopbackSources().then(v => devices.setLoopbacks(v)).catch(() => {});
    listMics().then(v => devices.setMics(v)).catch(() => {});
  }

  function handleCoreEvent(ev: CoreEvent) {
    switch (ev.kind) {
      case 'session-started': session.start(ev.session_id, Date.now()); break;
      case 'session-stopped': session.stop(); listSessions().then(v => sessions = v); break;
      case 'transcript-live':
        transcript.live({ status: ev.status, text: ev.text, language: ev.language, chip: ev.chip }); break;
      case 'transcript-final':
        transcript.final({ status: ev.status, text: ev.text, language: ev.language, chip: ev.chip, ts_ms: ev.ts_ms }); break;
      case 'connection-state':
        session.setConnection(ev.state, ev.attempt, ev.retry_in_ms); break;
      case 'latency': session.setLatency(ev.median_ms); break;
    }
  }

  onMount(() => {
    let unlisten: (() => void) | undefined;
    const ro = new ResizeObserver((entries) => {
      const first = entries[0];
      if (first) mainWidth = Math.round(first.contentRect.width);
    });
    ro.observe(document.body);
    const tick = setInterval(() => {
      if (session.recording && session.startedAt) elapsedMs = Date.now() - session.startedAt;
    }, 250);

    (async () => {
      const cfg = await getConfig();
      config.setConfig(cfg);
      applyTheme(cfg.theme);
      config.setHasApiKey(await hasApiKey(config.apiKeyAccount));
      sessions = await listSessions();
      refreshSources();
      unlisten = await onCoreEvent(handleCoreEvent);
    })();

    return () => { unlisten?.(); ro.disconnect(); clearInterval(tick); };
  });

  async function onSearch(q: string) {
    query = q;
    if (!q.trim()) { searchHits = []; return; }
    const hits = await searchTranscripts(q);
    const matchIds = new Set(hits.map(h => h.session_id));
    searchHits = sessions.filter(s => matchIds.has(s.id));
  }

  async function onStart() {
    if (!config.hasApiKey || !config.config || !selectedSource) return;
    transcript.reset();
    try {
      await startSession({
        mode,
        language_a: config.config.language_a,
        language_b: config.config.language_b,
        mine: config.config.mine,
        device_id: selectedSource.id,
        api_key_account: config.apiKeyAccount,
      });
      permissionKind = null;
    } catch (e) {
      const msg = String(e).toLowerCase();
      if (msg.includes('mic') || msg.includes('microphone')) permissionKind = 'mic';
      else if (msg.includes('audio capture') || msg.includes('sckit')) permissionKind = 'audio-capture';
      else throw e;
    }
  }

  async function onStop()       { await stopSession(); }
  async function onSwap()       {
    const c = config.config!;
    const next = { ...c, mine: (c.mine === 'a' ? 'b' : 'a') as WhichLang };
    await setConfig(next);
    config.setConfig(next);
  }
  async function onOverlayToggle() {
    if (overlayShown) { await hideOverlay(); overlayShown = false; }
    else              { await showOverlay(); overlayShown = true;  }
  }
  function onSettings() { settingsOpen = true; }

  const summary = $derived(mode === 'meeting'
    ? `one_way → ${langB.code}` : `two_way · ${langA.code} ⇄ ${langB.code}`);
</script>

<VoxWindow>
  <Toolbar
    {mode} onmode={(m) => mode = m}
    recording={session.recording}
    onstart={onStart} onstop={onStop}
    onsettings={onSettings} onoverlay={onOverlayToggle}
    overlayShown={overlayShown}
    a={langA} b={langB} {mine}
    onswap={onSwap}
    source={selectedSource}
    sourceOptions={mode === 'meeting' ? meetingSources : micSources}
    onsource={(d) => selectedSource = d} />

  <PermissionBanner kind={permissionKind} ondismiss={() => permissionKind = null} />

  <div class="flex-1 flex overflow-hidden">
    <Sidebar
      sessions={query ? searchHits : sessions}
      activeId={session.sessionId}
      onselect={() => {}}
      onsearch={onSearch}
      query={query} />

    <div class="flex-1 flex flex-col min-w-0">
      {#if !config.hasApiKey}
        <NoApiKey onaddkey={() => settingsOpen = true} />
      {:else if !session.recording && transcript.original.length === 0 && transcript.translation.length === 0}
        <EmptyState {mode} />
      {:else}
        <TranscriptPane
          {mode} a={langA} b={langB} {mine}
          original={transcript.original}
          translation={transcript.translation}
          liveOriginal={transcript.liveOriginal}
          liveTranslation={transcript.liveTranslation} />
      {/if}
      <StatusBar
        recording={session.recording}
        elapsedMs={elapsedMs} levelDb={-18}
        latencyMs={session.latencyMs}
        {mode}
        translationSummary={summary}
        model="stt-rt-v4"
        audioFormat="16 kHz · mono · s16le"
        width={mainWidth - 240} />
    </div>
  </div>
</VoxWindow>

<SettingsSheet open={settingsOpen} onclose={() => settingsOpen = false} />
