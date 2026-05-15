<script lang="ts">
  import ReadingControls from '../transcript/ReadingControls.svelte';
  import type { AppConfig } from '../../types';
  interface Props { cfg: AppConfig; onchange: (next: AppConfig) => void }
  const { cfg, onchange }: Props = $props();
</script>

<section class="pb-1">
  <div class="text-[12px] font-semibold mb-2" style:color="var(--vt-text)">Reading</div>
  <ReadingControls cfg={cfg} onchange={onchange} />
</section>
