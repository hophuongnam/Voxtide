# Mid-Session Context Switch — Design Spec

**Date:** 2026-07-01 · **Status:** Approved, ready to implement · **Scope:** Desktop (builds on the context-presets feature shipped in v0.1.16)

## Problem

Context is fixed for a whole recording: `SonioxBYOK` captures `context` once at `open()` and re-sends it unchanged on every reconnect (`soniox.rs:192,233`), and the toolbar `ContextPicker` is disabled while recording. Users want to switch the active context mid-recording (topic shifts mid-meeting).

Soniox accepts `context` **only** in the initial config frame — the sole mid-stream control frames are `finalize`/`keepalive` (verified: soniox.com/docs/stt/api-reference/websocket-api). So a mid-session change **requires reopening the WebSocket** with the new context. We do this **in place, within the same session/transcript** — not a session restart.

## Approved approach & UX

Un-disable the picker while recording. Picking a context mid-session triggers a **reconnect-with-new-context** on the existing worker reconnect loop, showing a brief **"applying context…"** status hint. **No transcript marker.**

**Cost accepted — describe accurately (do not understate):** reconnecting closes the socket, which discards the **entire unfinalized tail since the last endpoint/final** — those in-progress words never finalize, because the worker does not replay already-sent audio on reconnect (`audio_ms_sent` resets to 0). This is NOT "just an ephemeral partial that gets replaced": normally partial→final, here partial→nothing. Already-committed finals are preserved. The loss is bounded by time-since-last-pause (usually small at a topic boundary), and it is the **identical** loss a mid-session *network* reconnect already incurs today — a known quantity, not a new hazard. **Guidance to surface to the user** (picker tooltip / one-liner): *switch at a natural pause for the cleanest result.*

## Control path (end to end)

```
ContextPicker (enabled while recording)
  → MainApp.onContextPick(id): persist({active_context_id}) + if recording, updateContext(resolvedText)
  → ipc updateContext(text) → Tauri command update_context(text)
  → SessionController.update_context(text)  [mpsc → its worker task]
  → worker branch: provider.set_context(text)     [new trait method, default no-op]
  → SonioxBYOK.set_context: audio_tx.send(Outbound::SetContext(text))
  → reconnect loop: context = text; emit ContextSwitching; break 'inner; reconnect immediately
  → build_initial_config(&cfg, &context) now carries the new context
```

## Rust changes

**`translation/mod.rs`:**
- `TranslationEvent`: add `ContextSwitching` (marker; no fields). Adding a variant forces every exhaustive match to add an arm — let the compiler enumerate them.
- `TranslationProvider` trait: add `async fn set_context(&mut self, text: String) -> Result<()> { Ok(()) }` — **default no-op** so the mock provider and all other impls/test sites are unaffected.

**`translation/soniox.rs`:**
- `enum Outbound` (soniox.rs:88): add `SetContext(String)`.
- In the spawned task, make the captured context mutable (`let mut context = …`, was `let context = self.context.clone();` at :192).
- In the inner-loop `audio_rx.recv()` arm (`:281`), handle `Outbound::SetContext(new)`: set `context = new`, `let _ = event_tx.send(TranslationEvent::ContextSwitching).await`, set a per-`'outer`-iteration `context_switch = true` flag, `break 'inner`.
- After the inner loop, **before** the `got_tokens`/`MAX_ATTEMPTS`/backoff block (`:433`): `if context_switch { attempt = 0; continue 'outer; }` — reconnect immediately, budget-neutral (the `attempt += 1` at `:202` lands it at 1), no backoff sleep, no `Reconnecting` event (the `ContextSwitching` event already fired).
- `SonioxBYOK::set_context`: mirror `send_audio` (:474) — `audio_tx.send(Outbound::SetContext(text))`, error if not open.

**`session.rs`:**
- `SessionController`: add an `update_context` mpsc (sender on the controller, set in `start()` / cleared on stop; receiver a new arm in the worker `select!`), following the existing `stop_audio`/stop-channel pattern. The worker arm calls `provider.set_context(text).await` (log-and-continue on error; a failed switch must not kill the session).
- `SessionController::update_context(&self, text: String)`: send on the mpsc; no-op if no active session.
- Event mapping (the `handle` fn, `:465`): add `TranslationEvent::ContextSwitching => CoreEvent::ConnectionState { state: "context-switching", attempt: None, retry_in_ms: None }`. (`Connected` → `state:"connected"` already clears it when the new socket is up.)

**`commands/lifecycle.rs`:** new `#[tauri::command] async fn update_context(text: String, state) -> Result<(), String>` → `state.controller.update_context(text)`. Register it in the invoke handler (lib.rs).

## Frontend changes

- **`ipc.ts`:** `export const updateContext = (text: string) => invoke('update_context', { text });`
- **`Toolbar.svelte` / `ContextPicker.svelte`:** the picker is **enabled while recording** (drop the `disabled={p.recording}` wiring; picker is always interactive).
- **`MainApp.svelte` `onContextPick(id)`:** keep `persist({ active_context_id: id })`; **additionally**, when `session.recording`, resolve the newly-picked text directly from the id (`config.config?.contexts.find(c => c.id === id)?.text ?? ''`) and call `updateContext(text)`. Stopped → persist only (applies at next start, unchanged).
- **`StatusBar`:** when the connection state is `"context-switching"`, show "applying context…" (mirror how `"reconnecting"` is displayed). Cleared by the next `"connected"`/token.

## Tasks (sequenced; each: Sonnet implements TDD, Opus reviews)

- **C1 — Soniox worker mid-session context.** `Outbound::SetContext`; `TranslationEvent::ContextSwitching`; `set_context` trait method (default no-op); mutable context + switch-triggered immediate, budget-neutral reconnect in the loop; `SonioxBYOK::set_context`. **Also add the `handle`-fn arm** `ContextSwitching → ConnectionState{state:"context-switching", attempt:None, retry_in_ms:None}` in `session.rs` — REQUIRED to keep the workspace compiling (a new `TranslationEvent` variant makes that exhaustive match non-exhaustive); this is C1's ONLY `session.rs` edit (the controller channel + command are C2). **Before writing worker code, confirm the test is expressible:** verify `tests/soniox_reconnect.rs`'s mock-WS harness can (i) inject a `SetContext` mid-stream and (ii) capture the **second** connection's init frame. If it can't do this cheaply, extend the harness FIRST — do not write a test that silently asserts nothing. Test: after a `SetContext`, the worker reconnects, the **2nd** connection's init config carries the **new** context, and the switch neither consumes the `MAX_ATTEMPTS` budget nor emits `Reconnecting` (only `ContextSwitching`).
- **C2 — Controller + command.** `SessionController.update_context` + control channel + worker `select!` arm calling `provider.set_context` (log-and-continue on error — a failed switch must not kill the session); `update_context` Tauri command + registration in `lib.rs`. (The `ContextSwitching → ConnectionState` mapping was done in C1.) Test: `update_context` routes to `provider.set_context` — the test mock must **override** the default-no-op `set_context` to record the call (otherwise nothing is observable); plus the `ContextSwitching → ConnectionState{state:"context-switching"}` mapping.
- **C3 — Frontend picker + wiring + status hint.** `ipc.updateContext`; picker enabled while recording; `onContextPick` calls `updateContext(resolvedText)` only when recording; StatusBar "applying context…" on the new state. Tests: picker interactive while recording; `onContextPick` calls `updateContext` with the picked text when recording and NOT when stopped.

## Edge cases

- **Switch to "No context"** mid-session → empty text → reconnect with no `context` section (identical to a no-context start).
- **Rapid switches** → one reconnect each, last context wins (no debounce — acceptable; note in code).
- **Switch during a network reconnect** → the mutable `context` is read at the next connect, so the new value is used.
- **`set_context` after EOS/stop** → the audio arm stops polling after `client_eos`; a late switch is ignored. Controller no-ops when no session is active.

## Out of scope

Transcript marker at the switch point; **finalize-then-reconnect** flush for guaranteed no-loss (deferred — the unfinalized-tail loss is accepted, matching existing reconnect behavior; revisit only if users report losing words mid-switch); Android (conversation-only, separate view); any debounce.

## Verification

`pnpm test` + `cargo test` green; desktop build; then on-device smoke: start recording, switch context mid-session, confirm the "applying context…" hint appears briefly and subsequent transcription reflects the new context (e.g. a name/jargon term only in the new preset gets recognized), and that a network blip still reconnects normally.
