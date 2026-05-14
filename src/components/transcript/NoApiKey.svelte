<script lang="ts">
  import Icon from '../icons/Icon.svelte';
  interface Props { onaddkey: () => void; }
  const { onaddkey }: Props = $props();
</script>

<div class="flex-1 flex flex-col items-center justify-center p-6 gap-3">
  <div class="w-14 h-14 rounded-xl flex items-center justify-center"
       style:background="var(--vt-warn-tint)" style:border="0.5px solid var(--vt-warn-border)"
       style:color="var(--vt-warn)"><Icon name="key" size={22} stroke={1.5} /></div>
  <div class="text-[15px] font-semibold" style:color="var(--vt-text)">Add your Soniox API key</div>
  <div class="text-[12.5px] max-w-[380px] text-center leading-relaxed"
       style:color="var(--vt-muted)">
    Voxtide uses your own Soniox key for real-time translation. The key is stored on this device and never leaves it.
  </div>
  <div class="flex gap-2 mt-2">
    <button type="button" onclick={onaddkey}
      class="px-3.5 py-2 rounded-md border-0 cursor-pointer text-xs font-semibold"
      style:background="var(--vt-accent)" style:color="var(--vt-accent-ink)">Add API key</button>
    <a href="https://soniox.com/" target="_blank" rel="noopener"
      class="px-3.5 py-2 rounded-md text-xs font-medium no-underline"
      style:background="transparent" style:color="var(--vt-muted)"
      style:border="0.5px solid var(--vt-border)">Get a Soniox key →</a>
  </div>
</div>
