<script lang="ts">
  import LevelMeter from './LevelMeter.svelte';
  import { formatElapsed } from '../../lib/format';

  interface Props {
    recording: boolean;
    elapsedMs: number;
    levelDb: number;
    latencyMs: number | null;
    mode: 'meeting' | 'conversation';
    translationSummary: string;
    model: string;
    audioFormat: string;
    width: number;
  }
  const p: Props = $props();
  const showFormat   = $derived(p.width >= 900);
  const showLatency  = $derived(p.width >= 700);
  const showModel    = $derived(p.width >= 580);
  const showSummary  = $derived(p.width >= 480);
</script>

<div class="h-7 flex items-center gap-3 px-3 whitespace-nowrap"
     style:border-top="0.5px solid var(--vt-border)" style:background="var(--vt-bg-deep)"
     style:font-family="'Geist Mono Variable', monospace"
     style:font-size="10.5px" style:color="var(--vt-subtle)" style:letter-spacing="0.2px">
  <div class="flex items-center gap-1.5">
    <span class="block w-[7px] h-[7px] rounded-full"
          style:background={p.recording ? 'var(--vt-rec)' : 'var(--vt-dim)'}
          style:box-shadow={p.recording ? '0 0 0 3px var(--vt-rec-glow)' : ''}></span>
    <span class="uppercase" style:color={p.recording ? 'var(--vt-text)' : 'var(--vt-subtle)'}>
      {p.recording ? 'REC' : 'IDLE'}
    </span>
    <span>{p.recording ? formatElapsed(p.elapsedMs) : '—'}</span>
  </div>
  <span style:color="var(--vt-dim)">│</span>
  <div class="flex items-center gap-2">
    <LevelMeter active={p.recording} />
    <span>{p.recording ? `${p.levelDb} dB` : ''}</span>
  </div>
  {#if showModel}<span style:color="var(--vt-dim)">│</span><span>SONIOX · {p.model}</span>{/if}
  {#if showLatency}
    <span style:color="var(--vt-dim)">│</span>
    <span>{p.latencyMs != null && p.recording ? `${p.latencyMs} ms` : 'ws idle'}</span>
  {/if}
  <div class="flex-1"></div>
  {#if showSummary}<span>{p.translationSummary}</span>{/if}
  {#if showFormat}<span style:color="var(--vt-dim)">│</span><span>{p.audioFormat}</span>{/if}
</div>
