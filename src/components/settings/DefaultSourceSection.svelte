<script lang="ts">
  import type { AppConfig } from '../../types';
  import { MODE_LABELS } from '../../lib/modes';
  import { listMics, listLoopbackSources, type DeviceEntry } from '../../lib/ipc';
  interface Props { cfg: AppConfig; onchange: (next: AppConfig) => void; }
  const { cfg, onchange }: Props = $props();
  let mics = $state<DeviceEntry[]>([]);
  let loops = $state<DeviceEntry[]>([]);
  $effect(() => {
    listMics().then(v => mics = v).catch(() => {});
    listLoopbackSources().then(v => loops = v).catch(() => {});
  });
</script>
<section class="pb-5 mb-5" style:border-bottom="0.5px solid var(--vt-border)">
  <div class="text-[12px] font-semibold mb-2" style:color="var(--vt-text)">Default audio source</div>
  <div class="flex flex-col gap-2">
    <label class="flex items-center gap-2 text-[12px]" style:color="var(--vt-text)">
      <span class="w-24" style:color="var(--vt-muted)">{MODE_LABELS.meeting}</span>
      <select class="flex-1 px-2 py-1.5 rounded text-[12px]"
              style:background="var(--vt-surface)" style:border="0.5px solid var(--vt-border)" style:color="var(--vt-text)"
              value={cfg.default_meeting_source ?? ''}
              onchange={(e) => onchange({ ...cfg, default_meeting_source: (e.target as HTMLSelectElement).value || null })}>
        <option value="">— None —</option>
        {#each loops as l}<option value={l.id}>{l.label}</option>{/each}
      </select>
    </label>
    <label class="flex items-center gap-2 text-[12px]" style:color="var(--vt-text)">
      <span class="w-24" style:color="var(--vt-muted)">{MODE_LABELS.conversation}</span>
      <select class="flex-1 px-2 py-1.5 rounded text-[12px]"
              style:background="var(--vt-surface)" style:border="0.5px solid var(--vt-border)" style:color="var(--vt-text)"
              value={cfg.default_mic ?? ''}
              onchange={(e) => onchange({ ...cfg, default_mic: (e.target as HTMLSelectElement).value || null })}>
        <option value="">— None —</option>
        {#each mics as m}<option value={m.id}>{m.label}</option>{/each}
      </select>
    </label>
  </div>
</section>
