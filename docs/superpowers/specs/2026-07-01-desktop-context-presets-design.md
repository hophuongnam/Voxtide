# Desktop Context Presets — Design Spec

**Date:** 2026-07-01 · **Status:** Approved, ready to implement · **Scope:** Desktop only (macOS/Windows)

## Problem

The Soniox "context" feature (free-text names/jargon/domain that biases recognition + translation) exists on desktop but is **hard to use**: it's a single global textarea buried in the Settings sheet (`ContextSection.svelte` → `AppConfig.context: String`), applied to every session. To change it you must open Settings, edit, blur, close — and it's one shared blob, so recurring scenarios (standup, client call, family) mean constant retyping.

The user has **a few recurring contexts** and wants to **save them once and pick one per session**.

## Approved approach

A small **library of saved contexts** + a **per-session picker on the main screen**.

- **Library:** named contexts saved in config, managed (add/edit/delete) in Settings.
- **Selection:** a compact picker on the main screen next to the source selector; the chosen context's text is sent at Start. Selection is remembered across relaunches.

## Data model

**Rust** (`crates/voxtide-core/src/config.rs`):
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextPreset {
    pub id: String,    // opaque, frontend-generated (crypto.randomUUID)
    pub name: String,  // user label, e.g. "Standup"
    pub text: String,  // the Soniox context.text payload
}

// added to AppConfig:
#[serde(default)] pub contexts: Vec<ContextPreset>,
#[serde(default)] pub active_context_id: Option<String>,
// context: String  — KEPT, untouched (Android still binds to it; migration source)
```
Both new fields `#[serde(default)]` so pre-existing `config.json` files load unchanged (empty library, no selection). `Default for AppConfig` adds `contexts: Vec::new(), active_context_id: None`.

**TypeScript** (`src/types.ts`): mirror `ContextPreset` and add `contexts: ContextPreset[]` + `active_context_id: string | null` to `AppConfig`.

## Wire path — UNCHANGED

The Rust start command and Soniox config are **not touched**. `lifecycle.rs:130` still does `SonioxBYOK::new().with_context(req.context)`; `build_initial_config` still emits `context.text`. The frontend resolves the active preset to a plain string and sends it in the existing `context` payload field. This is why the whole feature is desktop-frontend + one config struct.

## Migration (frontend, desktop-only)

Desktop mounts `MainApp`; Android mounts `FaceToFaceView` (`main.ts:11`). So seeding lives in `MainApp` boot and **never runs on Android** — Android keeps using `cfg.context` directly.

On boot, if `contexts` is empty **and** legacy `cfg.context` is non-empty: create one preset (`name: "My context"`, `text: cfg.context`), set it active, **clear `cfg.context`**, and persist via `setConfig`. One-time and idempotent (once `contexts` is non-empty it never re-seeds; clearing `context` prevents a delete-all resurrection). Clearing is safe: desktop and Android configs are separate files on separate devices.

## Pure helpers (the money paths — unit-tested)

New `src/lib/context.ts`:
- `resolveActiveContext(cfg): string` — returns the active preset's `text`, or `''` if `active_context_id` is null/unknown (covers a deleted-active preset).
- `seedContextFromLegacy(cfg): AppConfig | null` — returns the migrated config if seeding is needed, else `null`.

## UI

**Settings — `ContextSection.svelte`** becomes a list editor over `cfg.contexts`:
```
Contexts
  Names, jargon, or domain — improves recognition and translation.
  ┌──────────────────────────────────────────┐
  │ [Standup            ]  🗑                  │
  │ [Speakers: Nam, Yuki. Standup.        ]   │
  ├──────────────────────────────────────────┤
  │ [Client Acme        ]  🗑                  │
  │ [Acme Corp. Topic: Q3 renewal.        ]   │
  └──────────────────────────────────────────┘
  [ + Add context ]
```
Name input + text textarea per row, blur-commit (existing pattern), delete button per row, "Add context" button. Each change calls `onchange` with the updated `contexts` array (persists immediately).

**Main screen — `MainApp.svelte`** gains a picker in the control row next to the source selector:
```
Source: [ System Audio ▾ ]   Context: [ Standup ▾ ]   ( ● Record )
                                        ├ No context
                                        ├ Standup
                                        ├ Client Acme
                                        └ Edit contexts…   → opens Settings sheet
```
- Bound to `active_context_id`; options = "No context" (null) + each preset (`id`→`name`) + "Edit contexts…".
- On change → `setConfig({ ...cfg, active_context_id })` immediately (persist-every-pick).
- **Disabled while recording** (a mid-session change would only apply at next Start; disabling avoids the false impression it took effect).
- Shows "No context" when `active_context_id` is null or points to a deleted preset.

**`onStart`:** change `context: config.config.context ?? ''` → `context: resolveActiveContext(config.config)`.

## Edge cases

- **No presets yet:** picker shows only "No context" + "Edit contexts…"; Start sends `''`.
- **Active preset deleted:** `resolveActiveContext` returns `''`; picker displays "No context".
- **Mid-recording change:** picker disabled during capture; applies at next Start.

## Scope / files

- `crates/voxtide-core/src/config.rs` — struct + fields + Default (+ round-trip test).
- `src/types.ts` — mirror types.
- `src/lib/context.ts` — `resolveActiveContext`, `seedContextFromLegacy` (+ unit tests).
- `src/components/settings/ContextSection.svelte` — list editor.
- `src/routes/MainApp.svelte` — picker, boot-seed, `onStart` resolve.

**Untouched:** `lifecycle.rs`, `soniox.rs`/`build_initial_config`, the start payload shape, `FaceToFaceView.svelte` and all Android behavior.

## Tasks (sequenced; each: Sonnet implements TDD, Opus reviews)

- **T1 — Rust config contract.** `ContextPreset` + `contexts`/`active_context_id` (serde default) + `Default`. Test: new fields round-trip; a `config.json` lacking them loads with empty library / `None`.
- **T2 — Frontend types + helpers.** `types.ts` mirror; `src/lib/context.ts` with both helpers. Unit tests for `resolveActiveContext` (active/null/unknown-id) and `seedContextFromLegacy` (seeds+clears once; no-op when library non-empty or legacy empty).
- **T3 — Settings list editor.** Rewrite `ContextSection.svelte` as add/edit/delete over `cfg.contexts`.
- **T4 — Main-screen picker + wiring.** Picker in `MainApp` (persist on change, disable during recording, "Edit contexts…" opens Settings), boot-seed via `seedContextFromLegacy`, `onStart` uses `resolveActiveContext`.

## Out of scope

Android (separate `context` field, unchanged), structured `terms`/`translation_terms` editor (YAGNI), any release/version bump (separate step, only on request).

## Verification

Full `pnpm test` + `cargo test` green, desktop build, then the mandatory 10-check smoke gate — with specific attention to: seed migrates the existing context once, picker persists across relaunch, selected context reaches Soniox at Start, deleting the active preset falls back to "No context".
