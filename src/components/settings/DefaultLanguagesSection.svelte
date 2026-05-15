<script lang="ts">
  import type { AppConfig } from '../../types';
  import { LANG_NAMES, LANG_CODES } from '../../lib/languages';
  interface Props { cfg: AppConfig; onchange: (next: AppConfig) => void; }
  const { cfg, onchange }: Props = $props();
  function pickCode(which: 'a' | 'b', code: string) {
    if (which === 'a' && code === cfg.language_b) return;
    if (which === 'b' && code === cfg.language_a) return;
    onchange(which === 'a' ? { ...cfg, language_a: code } : { ...cfg, language_b: code });
  }
</script>

<section class="pb-5 mb-5" style:border-bottom="0.5px solid var(--vt-border)">
  <div class="text-[12px] font-semibold mb-2" style:color="var(--vt-text)">Default languages</div>
  <div class="flex gap-3">
    <div class="p-3 rounded-md flex-1"
         style:background="var(--vt-surface)"
         style:border="0.5px solid var(--vt-border)">
      <div class="text-[10px] uppercase tracking-wide mb-1" style:color="var(--vt-subtle)">Source language</div>
      <select aria-label="Source language code"
              class="w-full text-[14px] bg-transparent border-0 outline-none cursor-pointer py-[2px] -ml-[2px]"
              style:color="var(--vt-text)"
              value={cfg.language_a}
              onchange={(e) => pickCode('a', e.currentTarget.value)}>
        {#each LANG_CODES as code}
          <option value={code} disabled={code === cfg.language_b}>{LANG_NAMES[code]}</option>
        {/each}
      </select>
    </div>
    <div class="p-3 rounded-md flex-1"
         style:background="var(--vt-surface)"
         style:border="0.5px solid var(--vt-border)">
      <div class="text-[10px] uppercase tracking-wide mb-1" style:color="var(--vt-subtle)">Target language</div>
      <select aria-label="Target language code"
              class="w-full text-[14px] bg-transparent border-0 outline-none cursor-pointer py-[2px] -ml-[2px]"
              style:color="var(--vt-text)"
              value={cfg.language_b}
              onchange={(e) => pickCode('b', e.currentTarget.value)}>
        {#each LANG_CODES as code}
          <option value={code} disabled={code === cfg.language_a}>{LANG_NAMES[code]}</option>
        {/each}
      </select>
    </div>
  </div>
</section>
