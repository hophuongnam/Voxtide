<script lang="ts">
  import type { AppConfig, ContextPreset } from '../../types';
  interface Props { cfg: AppConfig; onchange: (next: AppConfig) => void }
  const { cfg, onchange }: Props = $props();

  // Uncontrolled fields: seed each input/textarea from its preset, read the
  // DOM on blur. One disk write per edit (not per keystroke), and no local
  // state to drift from cfg.contexts. Rebuilds `contexts` immutably so each
  // commit is a single `onchange` carrying only that preset's change.
  function commitName(id: string, e: FocusEvent) {
    const value = (e.currentTarget as HTMLInputElement).value.trim();
    const preset = cfg.contexts.find((p) => p.id === id);
    if (!preset || value === preset.name) return;
    onchange({ ...cfg, contexts: cfg.contexts.map((p) => (p.id === id ? { ...p, name: value } : p)) });
  }

  function commitText(id: string, e: FocusEvent) {
    const value = (e.currentTarget as HTMLTextAreaElement).value.trim();
    const preset = cfg.contexts.find((p) => p.id === id);
    if (!preset || value === preset.text) return;
    onchange({ ...cfg, contexts: cfg.contexts.map((p) => (p.id === id ? { ...p, text: value } : p)) });
  }

  function addContext() {
    const preset: ContextPreset = { id: crypto.randomUUID(), name: '', text: '' };
    onchange({ ...cfg, contexts: [...cfg.contexts, preset] });
  }

  function removeContext(id: string) {
    onchange({ ...cfg, contexts: cfg.contexts.filter((p) => p.id !== id) });
  }
</script>

<section class="pb-5 mb-5" style:border-bottom="0.5px solid var(--vt-border)">
  <div class="text-[12px] font-semibold mb-2" style:color="var(--vt-text)">Contexts</div>
  <div class="text-[11px] mb-2" style:color="var(--vt-muted)">
    Names, jargon, or domain — improves recognition and translation.
  </div>
  {#each cfg.contexts as preset (preset.id)}
    <div class="mb-3">
      <div class="flex items-center gap-2">
        <input value={preset.name} onblur={(e) => commitName(preset.id, e)}
               aria-label="Context name" placeholder="Name"
               class="flex-1 px-3 py-2 rounded text-[12px] outline-none"
               style:background="var(--vt-surface)" style:color="var(--vt-text)"
               style:border="0.5px solid var(--vt-border)" />
        <button type="button" onclick={() => removeContext(preset.id)}
                aria-label="Delete context"
                class="shrink-0 px-1 bg-transparent border-0 cursor-pointer text-[13px]"
                style:color="var(--vt-muted)">🗑</button>
      </div>
      <textarea value={preset.text} onblur={(e) => commitText(preset.id, e)} rows="2"
                aria-label="Context text"
                placeholder="e.g. Speakers: Nam, Yuki. Company: Acme."
                class="w-full mt-2 px-3 py-2 rounded text-[12px] outline-none resize-y"
                style:background="var(--vt-surface)" style:color="var(--vt-text)"
                style:border="0.5px solid var(--vt-border)"></textarea>
    </div>
  {/each}
  <button type="button" onclick={addContext}
          class="px-3 py-1.5 rounded text-[11px] cursor-pointer"
          style:background="transparent" style:color="var(--vt-muted)"
          style:border="0.5px solid var(--vt-border)">+ Add context</button>
</section>
