<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { AppConfig } from '../../types';
  import ReadingControls from './ReadingControls.svelte';
  interface Props {
    label: string;
    code: string;
    sub?: string;
    accent?: boolean;
    children: Snippet;
    bodyRef?: (el: HTMLElement | null) => void;
    cfg?: AppConfig | null;
    onconfigchange?: (next: AppConfig) => void;
  }
  const {
    label, code, sub = '', accent = false, children, bodyRef,
    cfg = null, onconfigchange = () => {},
  }: Props = $props();
  let body: HTMLElement | undefined = $state();
  $effect(() => { bodyRef?.(body ?? null); return () => bodyRef?.(null); });

  let rcOpen = $state(false);
  let rcWrap: HTMLElement | undefined = $state();
  function onWinClick(e: MouseEvent) {
    if (rcOpen && rcWrap && !rcWrap.contains(e.target as Node)) rcOpen = false;
  }
</script>

<svelte:window
  onkeydown={(e) => { if (e.key === 'Escape') rcOpen = false; }}
  onclick={onWinClick} />

<div class="flex-1 flex flex-col min-w-0">
  <div class="h-[38px] px-4 flex items-center gap-2"
       style:border-bottom="0.5px solid var(--vt-border)" style:background="var(--vt-bg)">
    <span class="text-[11px] font-semibold" style:color="var(--vt-text)">{label}</span>
    <span class="px-1.5 py-[2px] rounded text-[9.5px] font-semibold tracking-wide"
          style:color={accent ? 'var(--vt-accent)' : 'var(--vt-muted)'}
          style:background={accent ? 'var(--vt-accent-tint-10)' : 'var(--vt-surface)'}
          style:border={`0.5px solid ${accent ? 'var(--vt-accent-tint-25)' : 'var(--vt-border)'}`}
          style:font-family="'Geist Mono Variable', monospace">{code}</span>
    {#if sub}<span class="text-[11px]" style:color="var(--vt-subtle)">{sub}</span>{/if}
    {#if cfg}
      <div class="flex-1"></div>
      <div class="relative" bind:this={rcWrap}>
        <button type="button" onclick={() => rcOpen = !rcOpen}
                class="px-1.5 py-[2px] rounded text-[10px] cursor-pointer"
                style:background="var(--vt-surface)" style:color="var(--vt-muted)"
                style:border="0.5px solid var(--vt-border)">Aa</button>
        {#if rcOpen}
          <div class="absolute right-0 mt-1 z-20 rounded-md"
               style:background="var(--vt-surface)" style:border="0.5px solid var(--vt-border)">
            <ReadingControls cfg={cfg} onchange={(n) => onconfigchange(n)} />
          </div>
        {/if}
      </div>
    {/if}
  </div>
  <div bind:this={body} class="flex-1 overflow-auto pt-1 pb-3">{@render children()}</div>
</div>
