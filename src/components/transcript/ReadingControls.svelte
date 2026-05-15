<script lang="ts">
  import type { AppConfig, FontSize } from '../../types';
  interface Props { cfg: AppConfig; onchange: (next: AppConfig) => void }
  const { cfg, onchange }: Props = $props();
  const sizes: FontSize[] = ['xs', 's', 'm', 'l', 'xl'];
  function setSize(s: FontSize) { onchange({ ...cfg, font_size: s }); }
  function togglePinyin() { onchange({ ...cfg, show_pinyin: !cfg.show_pinyin }); }
</script>

<div class="p-2 w-[208px]">
  <div class="text-[11px] font-semibold mb-1.5" style:color="var(--vt-text)">Text size</div>
  <div class="inline-flex p-[2px] rounded-md mb-2.5"
       style:background="var(--vt-surface)" style:border="0.5px solid var(--vt-border)">
    {#each sizes as s}
      {@const active = cfg.font_size === s}
      <button type="button" onclick={() => setSize(s)} aria-pressed={active}
              class="px-2 py-1 rounded text-[10px] cursor-pointer border-0 uppercase"
              style:background={active ? 'var(--vt-surface3)' : 'transparent'}
              style:color={active ? 'var(--vt-text)' : 'var(--vt-muted)'}>{s}</button>
    {/each}
  </div>
  <button type="button" onclick={togglePinyin} aria-pressed={cfg.show_pinyin}
          class="flex items-center justify-between w-full px-2 py-1.5 rounded text-[11px] cursor-pointer border-0"
          style:background="transparent" style:color="var(--vt-text)">
    <span>拼 Show pinyin</span>
    <span class="px-1.5 py-[1px] rounded text-[9px] font-semibold"
          style:background={cfg.show_pinyin ? 'var(--vt-accent-tint-10)' : 'var(--vt-surface)'}
          style:color={cfg.show_pinyin ? 'var(--vt-accent)' : 'var(--vt-muted)'}
    >{cfg.show_pinyin ? 'ON' : 'OFF'}</span>
  </button>
</div>
