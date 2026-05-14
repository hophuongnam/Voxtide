<script lang="ts">
  import { onDestroy } from 'svelte';
  interface Props { active: boolean; }
  const { active }: Props = $props();
  let tick = $state(0);
  let id: any;
  $effect(() => {
    if (active) id = setInterval(() => tick++, 80);
    return () => clearInterval(id);
  });
  const bars = $derived(Array.from({ length: 14 }, (_, i) => {
    if (!active) return 0.12;
    const v = 0.25 + 0.5 * Math.abs(Math.sin((tick + i * 1.3) * 0.6 + i));
    return Math.min(1, v * (0.6 + 0.4 * Math.sin(tick * 0.2 + i * 0.4)));
  }));
  onDestroy(() => clearInterval(id));
</script>

<div class="inline-flex items-center gap-[2px] h-3.5">
  {#each bars as h}
    <div class="w-[2px] rounded-[1px]"
         style:height={`${100 * h}%`} style:min-height="2px"
         style:background={active ? (h > 0.7 ? 'var(--vt-accent)' : 'var(--vt-accent-dim)') : 'var(--vt-dim)'}
         style:transition="height .08s"></div>
  {/each}
</div>
