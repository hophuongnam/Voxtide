<script lang="ts">
  import type { Snippet } from 'svelte';
  interface Props { label: string; code: string; sub?: string; accent?: boolean; children: Snippet; }
  const { label, code, sub = '', accent = false, children }: Props = $props();
</script>

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
  </div>
  <div class="flex-1 overflow-auto pt-1 pb-3">{@render children()}</div>
</div>
