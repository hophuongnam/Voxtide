<script lang="ts">
  import type { AppConfig, WhichLang } from '../../types';
  interface Props { cfg: AppConfig; onchange: (next: AppConfig) => void; }
  const { cfg, onchange }: Props = $props();
  const LANG_NAMES: Record<string, string> = { en: 'English', vi: 'Vietnamese', ja: 'Japanese', ko: 'Korean', es: 'Spanish', de: 'German', fr: 'French', zh: 'Chinese' };
  function pickMine(which: WhichLang) { onchange({ ...cfg, mine: which }); }
</script>
<section class="pb-5 mb-5" style:border-bottom="0.5px solid var(--vt-border)">
  <div class="text-[12px] font-semibold mb-2" style:color="var(--vt-text)">Default languages</div>
  <div class="flex gap-3">
    <button type="button" class="text-left p-3 rounded-md cursor-pointer flex-1"
            style:background={cfg.mine === 'a' ? 'var(--vt-surface3)' : 'var(--vt-surface)'}
            style:border={`0.5px solid ${cfg.mine === 'a' ? 'var(--vt-border-hi)' : 'var(--vt-border)'}`}
            onclick={() => pickMine('a')}>
      <div class="text-[10px] uppercase tracking-wide mb-1" style:color="var(--vt-subtle)">Language A</div>
      <div class="text-[14px]" style:color="var(--vt-text)">{LANG_NAMES[cfg.language_a] ?? cfg.language_a}</div>
      {#if cfg.mine === 'a'}<span class="mt-2 inline-block px-1.5 py-[1px] rounded text-[9px] font-bold tracking-wider"
                                  style:background="var(--vt-accent)" style:color="var(--vt-accent-ink)">MY LANGUAGE</span>{/if}
    </button>
    <button type="button" class="text-left p-3 rounded-md cursor-pointer flex-1"
            style:background={cfg.mine === 'b' ? 'var(--vt-surface3)' : 'var(--vt-surface)'}
            style:border={`0.5px solid ${cfg.mine === 'b' ? 'var(--vt-border-hi)' : 'var(--vt-border)'}`}
            onclick={() => pickMine('b')}>
      <div class="text-[10px] uppercase tracking-wide mb-1" style:color="var(--vt-subtle)">Language B</div>
      <div class="text-[14px]" style:color="var(--vt-text)">{LANG_NAMES[cfg.language_b] ?? cfg.language_b}</div>
      {#if cfg.mine === 'b'}<span class="mt-2 inline-block px-1.5 py-[1px] rounded text-[9px] font-bold tracking-wider"
                                  style:background="var(--vt-accent)" style:color="var(--vt-accent-ink)">MY LANGUAGE</span>{/if}
    </button>
  </div>
</section>
