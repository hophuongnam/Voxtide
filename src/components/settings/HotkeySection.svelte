<script lang="ts">
  import type { AppConfig } from '../../types';
  interface Props { cfg: AppConfig; onchange: (next: AppConfig) => void; }
  const { cfg, onchange }: Props = $props();
</script>
<section class="pb-5 mb-5" style:border-bottom="0.5px solid var(--vt-border)">
  <div class="text-[12px] font-semibold mb-2" style:color="var(--vt-text)">Global hotkey</div>
  <input type="text" value={cfg.hotkey}
         oninput={(e) => onchange({ ...cfg, hotkey: (e.target as HTMLInputElement).value })}
         class="px-3 py-2 rounded text-[12px] outline-none w-44"
         style:background="var(--vt-surface)" style:color="var(--vt-text)"
         style:border="0.5px solid var(--vt-border)" />
</section>
