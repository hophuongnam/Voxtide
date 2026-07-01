import type { AppConfig } from '../types';

/** Resolve the currently active context preset to the plain text Soniox
 *  expects. Returns `''` when `active_context_id` is `null`, or when it names
 *  a preset no longer present in `cfg.contexts` (the active preset was
 *  deleted) — both cases mean "no context". Not trimmed; Rust trims the
 *  payload downstream. */
export function resolveActiveContext(cfg: AppConfig): string {
  if (cfg.active_context_id === null) return '';
  const active = cfg.contexts.find((preset) => preset.id === cfg.active_context_id);
  return active?.text ?? '';
}

/** One-time migration from the legacy single-blob `cfg.context` to the
 *  preset library. If a library already exists, or there's no legacy text
 *  to migrate, this is a no-op (`null`) — safe to call unconditionally on
 *  every boot. Otherwise returns a new config with one seeded preset
 *  ("My context", holding the original `cfg.context` verbatim), that preset
 *  set active, and the legacy field cleared. Idempotent: once `contexts` is
 *  non-empty it never re-seeds, and clearing `context` prevents a
 *  delete-all-presets resurrection on a later boot. */
export function seedContextFromLegacy(cfg: AppConfig): AppConfig | null {
  if (cfg.contexts.length > 0) return null;
  if (cfg.context.trim() === '') return null;

  const seeded = { id: crypto.randomUUID(), name: 'My context', text: cfg.context };
  return {
    ...cfg,
    contexts: [seeded],
    active_context_id: seeded.id,
    context: '',
  };
}
