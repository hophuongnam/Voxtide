<script lang="ts">
  import type { AppConfig } from '../../types';
  interface Props { cfg: AppConfig; onchange: (next: AppConfig) => void }
  const { cfg, onchange }: Props = $props();
  // Uncontrolled textarea: seed the initial value from cfg, read the DOM on
  // blur. One disk write per edit (not per keystroke), and no local state to
  // drift from cfg.context.
  function commit(e: FocusEvent) {
    const next = (e.currentTarget as HTMLTextAreaElement).value.trim();
    if (next !== (cfg.context ?? '')) onchange({ ...cfg, context: next });
  }
</script>

<section class="pb-5 mb-5" style:border-bottom="0.5px solid var(--vt-border)">
  <div class="text-[12px] font-semibold mb-2" style:color="var(--vt-text)">Context</div>
  <label class="text-[11px] block" style:color="var(--vt-muted)">
    Names, jargon, or domain — improves recognition and translation. Optional.
    <textarea value={cfg.context ?? ''} onblur={commit} rows="3"
              placeholder="e.g. Speakers: Nam, Yuki. Company: Acme. Topic: quarterly review."
              class="w-full mt-2 px-3 py-2 rounded text-[12px] outline-none resize-y"
              style:background="var(--vt-surface)" style:color="var(--vt-text)"
              style:border="0.5px solid var(--vt-border)"></textarea>
  </label>
</section>
