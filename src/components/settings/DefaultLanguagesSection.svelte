<script lang="ts">
  import type { AppConfig, WhichLang } from '../../types';
  import { LANG_NAMES, LANG_CODES } from '../../lib/languages';
  interface Props { cfg: AppConfig; onchange: (next: AppConfig) => void; }
  const { cfg, onchange }: Props = $props();
  function pickMine(which: WhichLang) { onchange({ ...cfg, mine: which }); }
  function pickCode(which: WhichLang, code: string) {
    if (which === 'a' && code === cfg.language_b) return;
    if (which === 'b' && code === cfg.language_a) return;
    onchange(which === 'a' ? { ...cfg, language_a: code } : { ...cfg, language_b: code });
  }
</script>

<section class="pb-5 mb-5" style:border-bottom="0.5px solid var(--vt-border)">
  <div class="text-[12px] font-semibold mb-2" style:color="var(--vt-text)">Default languages</div>
  <div class="flex gap-3">
    <div class="p-3 rounded-md flex-1"
         style:background={cfg.mine === 'a' ? 'var(--vt-surface3)' : 'var(--vt-surface)'}
         style:border={`0.5px solid ${cfg.mine === 'a' ? 'var(--vt-border-hi)' : 'var(--vt-border)'}`}>
      <div class="text-[10px] uppercase tracking-wide mb-1" style:color="var(--vt-subtle)">Language A</div>
      <select aria-label="Language A code"
              class="w-full text-[14px] bg-transparent border-0 outline-none cursor-pointer mb-2 py-[2px] -ml-[2px]"
              style:color="var(--vt-text)"
              value={cfg.language_a}
              onchange={(e) => pickCode('a', e.currentTarget.value)}>
        {#each LANG_CODES as code}
          <option value={code} disabled={code === cfg.language_b}>{LANG_NAMES[code]}</option>
        {/each}
      </select>
      <button type="button"
              aria-pressed={cfg.mine === 'a'}
              class="inline-block px-1.5 py-[2px] rounded text-[9px] font-bold tracking-wider cursor-pointer border-0"
              style:background={cfg.mine === 'a' ? 'var(--vt-accent)' : 'var(--vt-surface3)'}
              style:color={cfg.mine === 'a' ? 'var(--vt-accent-ink)' : 'var(--vt-muted)'}
              onclick={() => pickMine('a')}>
        {cfg.mine === 'a' ? 'MY LANGUAGE' : 'Make mine'}
      </button>
    </div>
    <div class="p-3 rounded-md flex-1"
         style:background={cfg.mine === 'b' ? 'var(--vt-surface3)' : 'var(--vt-surface)'}
         style:border={`0.5px solid ${cfg.mine === 'b' ? 'var(--vt-border-hi)' : 'var(--vt-border)'}`}>
      <div class="text-[10px] uppercase tracking-wide mb-1" style:color="var(--vt-subtle)">Language B</div>
      <select aria-label="Language B code"
              class="w-full text-[14px] bg-transparent border-0 outline-none cursor-pointer mb-2 py-[2px] -ml-[2px]"
              style:color="var(--vt-text)"
              value={cfg.language_b}
              onchange={(e) => pickCode('b', e.currentTarget.value)}>
        {#each LANG_CODES as code}
          <option value={code} disabled={code === cfg.language_a}>{LANG_NAMES[code]}</option>
        {/each}
      </select>
      <button type="button"
              aria-pressed={cfg.mine === 'b'}
              class="inline-block px-1.5 py-[2px] rounded text-[9px] font-bold tracking-wider cursor-pointer border-0"
              style:background={cfg.mine === 'b' ? 'var(--vt-accent)' : 'var(--vt-surface3)'}
              style:color={cfg.mine === 'b' ? 'var(--vt-accent-ink)' : 'var(--vt-muted)'}
              onclick={() => pickMine('b')}>
        {cfg.mine === 'b' ? 'MY LANGUAGE' : 'Make mine'}
      </button>
    </div>
  </div>
</section>
