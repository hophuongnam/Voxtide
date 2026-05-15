<script lang="ts">
  import type { AppConfig, Theme } from '../../types';
  import { applyTheme } from '../../theme/theme';
  interface Props { cfg: AppConfig; onchange: (next: AppConfig) => void; }
  const { cfg, onchange }: Props = $props();
  const opts: Theme[] = ['light', 'dark', 'system'];
  function set(t: Theme) { applyTheme(t); onchange({ ...cfg, theme: t }); }
</script>
<section class="pb-1">
  <div class="text-[12px] font-semibold mb-2" style:color="var(--vt-text)">Appearance</div>
  <div class="inline-flex p-[2px] rounded-md"
       style:background="var(--vt-surface)" style:border="0.5px solid var(--vt-border)">
    {#each opts as t}
      {@const active = cfg.theme === t}
      <button type="button" onclick={() => set(t)} aria-pressed={active}
              class="px-3 py-1.5 rounded text-[11px] cursor-pointer border-0 capitalize"
              style:background={active ? 'var(--vt-surface3)' : 'transparent'}
              style:color={active ? 'var(--vt-text)' : 'var(--vt-muted)'}>{t}</button>
    {/each}
  </div>
</section>
