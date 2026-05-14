<script lang="ts">
  import ApiKeySection from './ApiKeySection.svelte';
  import DefaultLanguagesSection from './DefaultLanguagesSection.svelte';
  import DefaultSourceSection from './DefaultSourceSection.svelte';
  import HotkeySection from './HotkeySection.svelte';
  import AppearanceSection from './AppearanceSection.svelte';
  import { getConfig, hasApiKey, setConfig } from '../../lib/ipc';
  import type { AppConfig } from '../../types';

  interface Props { open: boolean; onclose: () => void; }
  const { open, onclose }: Props = $props();
  let cfg = $state<AppConfig | null>(null);
  let keyPresent = $state(false);
  const account = 'default';

  async function reload() {
    cfg = await getConfig();
    keyPresent = await hasApiKey(account);
  }

  $effect(() => { if (open) reload(); });

  async function onChange(next: AppConfig) {
    cfg = next;
    await setConfig(next);
  }
</script>

{#if open && cfg}
  <div role="presentation" class="fixed inset-0 z-50 flex items-center justify-center" style:background="rgba(0,0,0,0.45)"
       onclick={onclose} onkeydown={(e) => e.key === 'Escape' && onclose()}>
    <div role="dialog" aria-modal="true" tabindex="-1" class="rounded-xl w-[560px] h-[680px] p-6 overflow-auto"
         style:background="var(--vt-bg)" style:border="0.5px solid var(--vt-border)"
         onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()}>
      <div class="flex items-center justify-between mb-4">
        <div class="text-[14px] font-semibold" style:color="var(--vt-text)">Settings</div>
        <button type="button" onclick={onclose} class="bg-transparent border-0 cursor-pointer text-[14px]"
                style:color="var(--vt-muted)">✕</button>
      </div>
      <ApiKeySection hasKey={keyPresent} {account} onsaved={() => reload()} />
      <DefaultLanguagesSection cfg={cfg} onchange={onChange} />
      <DefaultSourceSection cfg={cfg} onchange={onChange} />
      <HotkeySection cfg={cfg} onchange={onChange} />
      <AppearanceSection cfg={cfg} onchange={onChange} />
    </div>
  </div>
{/if}
