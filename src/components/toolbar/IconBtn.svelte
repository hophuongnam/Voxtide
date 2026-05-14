<script lang="ts">
  import Icon from '../icons/Icon.svelte';
  import type { IconName } from '../icons/paths';
  interface Props { name: IconName; active?: boolean; title?: string; onclick?: () => void; }
  const { name, active = false, title, onclick }: Props = $props();
</script>
<button type="button" {title} onclick={onclick}
  class="w-[30px] h-[30px] rounded-md cursor-pointer inline-flex items-center justify-center"
  style:background={active ? 'var(--vt-surface3)' : 'var(--vt-surface)'}
  style:border={`0.5px solid ${active ? 'var(--vt-border-hi)' : 'var(--vt-border)'}`}
  style:color={active ? 'var(--vt-text)' : 'var(--vt-muted)'}>
  <Icon {name} size={14} />
</button>
