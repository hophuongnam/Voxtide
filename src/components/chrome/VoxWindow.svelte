<script lang="ts">
  import type { Snippet } from 'svelte';
  interface Props { children: Snippet; }
  const { children }: Props = $props();
</script>

<div class="w-full h-full flex flex-col" style:background="var(--vt-bg)" style:color="var(--vt-text)"
     style:font-family="'Geist Variable', system-ui, sans-serif">
  {@render children()}
</div>
