<script lang="ts">
  import { setApiKey, clearApiKey } from '../../lib/ipc';
  interface Props { hasKey: boolean; account: string; onsaved: () => void; }
  const { hasKey, account, onsaved }: Props = $props();
  let value = $state('');
  async function save() {
    if (!value.trim()) return;
    await setApiKey(account, value.trim());
    value = '';
    onsaved();
  }
  async function clear() { await clearApiKey(account); onsaved(); }
</script>

<section class="pb-5 mb-5" style:border-bottom="0.5px solid var(--vt-border)">
  <div class="text-[12px] font-semibold mb-2" style:color="var(--vt-text)">Soniox API key</div>
  <label class="text-[11px] block mb-2" style:color="var(--vt-muted)">
    <span class="sr-only">Soniox API key</span>
    <input type="password" placeholder={hasKey ? '••••••••' : 'sk_live_…'}
           aria-label="Soniox API key"
           value={value}
           oninput={(e) => value = (e.target as HTMLInputElement).value}
           class="w-full px-3 py-2 rounded text-[12px] outline-none"
           style:background="var(--vt-surface)" style:color="var(--vt-text)"
           style:border="0.5px solid var(--vt-border)" />
  </label>
  <div class="flex gap-2">
    <button type="button" onclick={save} class="px-3 py-1.5 rounded text-[11px] font-semibold cursor-pointer"
            style:background="var(--vt-accent)" style:color="var(--vt-accent-ink)" style:border="none">Save</button>
    {#if hasKey}
      <button type="button" onclick={clear} class="px-3 py-1.5 rounded text-[11px] cursor-pointer"
              style:background="transparent" style:color="var(--vt-muted)"
              style:border="0.5px solid var(--vt-border)">Remove</button>
    {/if}
  </div>
</section>
