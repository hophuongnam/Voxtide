<script lang="ts">
  interface Props { open: boolean; onclose: () => void; }
  const { open, onclose }: Props = $props();
</script>

{#if open}
  <div class="fixed inset-0 z-50 flex items-center justify-center" style:background="rgba(0,0,0,0.45)" onclick={onclose}>
    <div class="rounded-xl w-[560px] h-[680px] p-5" style:background="var(--vt-bg)"
         style:border="0.5px solid var(--vt-border)" onclick={(e) => e.stopPropagation()}>
      <div class="text-sm font-semibold" style:color="var(--vt-text)">Settings (filled in Task 14)</div>
    </div>
  </div>
{/if}
