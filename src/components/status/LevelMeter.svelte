<script lang="ts">
  // Purely decorative activity meter. CSS keyframes (staggered by a negative
  // per-bar delay) replace the old 80ms setInterval + per-tick sin() rebuild
  // of all 14 bars — zero JS work per frame, and no timer to leak.
  interface Props { active: boolean; }
  const { active }: Props = $props();
</script>

<div class="inline-flex items-center gap-[2px] h-3.5">
  {#each Array.from({ length: 14 }) as _, i (i)}
    <div class={`w-[2px] rounded-[1px] ${active ? 'vt-bar' : ''}`}
         style:--i={i}
         style:height={active ? undefined : '12%'}
         style:min-height="2px"
         style:background={active ? 'var(--vt-accent-dim)' : 'var(--vt-dim)'}></div>
  {/each}
</div>

<style>
  .vt-bar {
    animation: vt-bar 1.2s ease-in-out infinite alternate;
    /* Negative delay staggers every bar's phase instantly. */
    animation-delay: calc(var(--i) * -80ms);
  }
  @keyframes vt-bar {
    from { height: 18%; }
    50%  { height: 85%; }
    to   { height: 38%; }
  }
</style>
