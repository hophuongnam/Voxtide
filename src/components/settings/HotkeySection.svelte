<script lang="ts">
  import type { AppConfig } from '../../types';
  interface Props { cfg: AppConfig; onchange: (next: AppConfig) => void | Promise<void>; }
  const { cfg, onchange }: Props = $props();
  let error = $state<string | null>(null);

  // Commit on change (Enter/blur), not per keystroke: an oninput persist
  // saved garbage like "Ctrl+Shif" mid-typing — and now that the hotkey is
  // registered live on save, each keystroke would also thrash the OS binding.
  async function commit(e: Event) {
    const hotkey = (e.target as HTMLInputElement).value.trim();
    error = null;
    try {
      await onchange({ ...cfg, hotkey });
    } catch (err) {
      const m = err as { message?: string } | string | null;
      error = typeof m === 'string' ? m : (m?.message ?? 'could not register hotkey');
    }
  }
</script>
<section class="pb-5 mb-5" style:border-bottom="0.5px solid var(--vt-border)">
  <div class="text-[12px] font-semibold mb-2" style:color="var(--vt-text)">Global hotkey</div>
  <input type="text" value={cfg.hotkey}
         onchange={commit}
         class="px-3 py-2 rounded text-[12px] outline-none w-44"
         style:background="var(--vt-surface)" style:color="var(--vt-text)"
         style:border="0.5px solid var(--vt-border)" />
  {#if error}
    <div class="text-[11px] mt-1" style:color="var(--vt-danger)">{error}</div>
  {/if}
</section>
