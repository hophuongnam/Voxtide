# Delete past transcript — Design Specification

**Status:** Approved (v1)
**Date:** 2026-05-14
**Author:** Nam Ho

## 1. Overview

Add the ability to delete a past transcript from Voxtide's history. The user mouses over a row in the sidebar's **History** list, clicks a small trash icon that reveals on hover, confirms in a modal sheet, and the session row plus all of its tokens (and the corresponding FTS5 entries) are removed from the local SQLite database. The currently-recording session is exempt from deletion at the UI level and at the backend level.

## 2. Goals & non-goals

### Goals
- Single-session delete from the **History** sidebar with one mouse path (hover → trash → Confirm).
- Atomic, irreversible: a single `DELETE FROM sessions WHERE id = ?` cascades through `tokens` (FK) and `tokens_fts` (trigger). No soft-delete column, no undo.
- Cannot accidentally delete the live capture: no trash affordance on the active row, and the backend command refuses the active session id.
- Consistent state after delete: sidebar list refreshes, search hits prune, past-session viewer clears if the deleted session was the one being viewed.

### Non-goals (v1)
- Bulk select / multi-delete / "delete all in this date group".
- Undo (would require either soft-delete column or in-memory token buffer).
- Keyboard shortcut (Delete/Backspace) on the sidebar list.
- Native OS confirm dialog (no `tauri-plugin-dialog` dependency).
- Right-click context menu (no context-menu primitive yet; not building one for this).
- Export-before-delete.

## 3. Interaction design

1. User hovers a row in the **History** list.
2. A `trash` icon fades in at the right edge of the row, vertically centered. It is shown **only** when `row.ended_at != null` (i.e., not the live capture).
3. Clicking the icon does **not** open the past-session viewer. The click handler calls `stopPropagation()` and emits `ondeleterequest(row)` upward.
4. `MainApp.svelte` opens a `ConfirmDeleteSheet` modal styled to match `SettingsSheet`. Body text composed from existing helpers:

   > **Delete this transcript?**
   > _`formatTime(row.started_at)` · `formatDuration(row.duration_ms ?? 0)` · `LANGA → LANGB`_
   > This cannot be undone.

   Buttons: **Cancel** (default focus, neutral, `var(--vt-muted)` text on transparent) and **Delete** (red destructive — see §5 on the `--vt-danger` token).
5. **Esc**, clicking outside the sheet, or clicking **Cancel** dismisses the modal without action.
6. Clicking **Delete** disables both buttons and the overlay/Esc dismissers (the modal becomes non-cancellable until the await resolves), calls `deleteSession(id)` via IPC, and on success closes the modal and updates UI state. On error, the sheet stays open and renders the error string under the buttons; both buttons and the dismissers re-enable.

## 4. Architecture & data flow

```
SessionItem  ──hover──▶  trash icon (only if row.ended_at != null)
   │
   └─ click (stopPropagation) ──▶ Sidebar.ondeleterequest(row)
                                      │
                                      ▼
                         MainApp: pendingDelete = row
                                      │
                                      ▼
                          ConfirmDeleteSheet (modal)
                                      │
                       ┌──────────────┴──────────────┐
                    Cancel                       Confirm
                       │                             │
                       ▼                             ▼
                  pending = null         deleteSession(id) via IPC
                                                     │
                                                     ▼
                            Tauri cmd: delete_session(state, id)
                              ├─ refuse if id == controller's active session
                              └─ Sessions::delete(pool, id)
                                  → DELETE FROM sessions WHERE id = ?
                                  → ON DELETE CASCADE removes tokens
                                  → tokens_ad trigger removes tokens_fts rows
                                                     │
                                                     ▼
                              MainApp: refresh sessions, prune searchHits,
                                       clear viewingId/pastOriginal/pastTranslation
                                       if id === viewingId
```

The cascade + FTS5 trigger chain is already exercised by
`crates/voxtide-core/tests/persistence_tokens.rs::deleting_session_cascades_to_tokens_and_fts`,
so the data layer needs no schema changes.

## 5. Components & files touched

### Backend (Rust)

| File | Change |
|---|---|
| `crates/voxtide-core/src/session.rs` | Store the live `session_id: i64` on `RunningSession` (currently it only holds `join` + the two stop channels). Add `SessionController::active_session_id(&self) -> Option<i64>` that returns `Some(id)` when the state is `Running` and `None` otherwise. |
| `crates/voxtide-core/src/persistence/sessions.rs` | Add `Sessions::delete(pool, id) -> Result<bool>` returning whether a row was deleted (false if not found). |
| `src-tauri/src/commands/sessions.rs` | Add `#[tauri::command] delete_session(state, id) -> Result<(), String>`. Call `state.controller.active_session_id()`; if it equals `Some(id)`, return `Err("cannot delete an active session")`. Otherwise call `Sessions::delete`. A missing row is **not** an error (idempotent — `Ok(())`). |
| `src-tauri/src/main.rs` | Register `commands::sessions::delete_session` in `tauri::generate_handler!`. |

### Frontend (Svelte / TypeScript)

| File | Change |
|---|---|
| `src/lib/ipc.ts` | Add `export const deleteSession = (id: number) => invoke<void>('delete_session', { id });` |
| `src/components/icons/paths.ts` | Add a `trash` glyph entry (currently absent from the `PATHS` const). A standard lid + bin path at the existing 24×24 viewBox. |
| `src/theme/theme.ts` (and any per-theme CSS) | Add a `--vt-danger` token (red, distinct from `--vt-rec` which is reserved for the live-recording dot). Used by the destructive Delete button. |
| `src/components/sidebar/SessionItem.svelte` | Render the trash Icon pinned to the right edge inside the existing row layout. Visible only on `:hover`/`:focus-within` of the row and only when `row.ended_at != null`. `onclick` calls `stopPropagation()` and `ondelete(row)`. |
| `src/components/sidebar/Sidebar.svelte` | Add `ondeleterequest: (row: SessionRow) => void` prop; forward `SessionItem`'s `ondelete` to it. |
| `src/components/sidebar/ConfirmDeleteSheet.svelte` (new, ~60 lines) | Modal sheet matching `SettingsSheet` styling. Props: `open`, `target: SessionRow \| null`, `busy: boolean`, `error: string \| null`, `onconfirm`, `oncancel`. Default focus on Cancel. Esc and overlay click route to `oncancel` **only when `!busy`**. |
| `src/routes/MainApp.svelte` | Add `pendingDelete: SessionRow \| null`, `deleting = false`, `deleteError: string \| null` state. Wire `Sidebar.ondeleterequest` to set `pendingDelete`. Render `<ConfirmDeleteSheet>` bound to it. On confirm: set `deleting = true`, `await deleteSession(id)`; on success: refresh `sessions`, prune `searchHits` by id, and if `viewingId === id` clear viewer state (`viewingId = null; pastOriginal = []; pastTranslation = [];`), then clear `pendingDelete`; on error: set `deleteError` and keep the sheet open; always clear `deleting` in `finally`. |

## 6. Error handling & edge cases

| Case | Behavior |
|---|---|
| Active session targeted | Tauri command rejects with `"cannot delete an active session"`. The modal renders the message under the buttons; Delete re-enables. UI also hides the trash icon on the active row, so this path is defense-in-depth. |
| Session vanished between list and delete | `Sessions::delete` returns `Ok(false)`; the Tauri command resolves `Ok(())` (idempotent). Frontend just refreshes the list. |
| DB error (locked, I/O) | Modal stays open and shows the error string under the buttons; Delete re-enables. No partial state — the SQL is a single atomic statement. |
| Deleting the viewed past session | Clear `viewingId`, `pastOriginal`, `pastTranslation`. The user lands on the live transcript pane (if recording) or `EmptyState`. |
| Deleting while a search filter is active | Prune the deleted id from `searchHits` immediately; also refresh `sessions` so re-clearing the search returns a coherent list. |
| Modal dismissed mid-flight (Esc / overlay click during await) | While `busy === true`, both buttons are disabled **and** the Esc and overlay-click handlers no-op. The modal becomes non-cancellable until the request resolves; the state then returns to either dismissed (success) or error-displayed (failure). |
| Keyboard | Esc cancels. **No** Delete/Backspace shortcut on the sidebar list in v1. |

## 7. Testing

### Rust — `crates/voxtide-core/tests/persistence_sessions.rs`
- `delete_removes_session_and_cascades` — create session + a few tokens, call `Sessions::delete`, assert `Sessions::list` is empty and `Tokens::search` returns nothing for the token text.
- `delete_missing_returns_false` — call `Sessions::delete` on a non-existent id; expect `Ok(false)`.

### Tauri — `src-tauri/tests/sessions_commands.rs` (new, mirroring `keychain_commands.rs`)
- Happy path: create a finished session, call `delete_session`, expect `Ok(())`, verify it's gone from `list_sessions`.
- Active-session refusal: drive the controller into a `Running` state by either (a) calling `start()` with a mock `AudioSource`/`TranslationProvider` pair from the existing `tests/audio_mock.rs` + `tests/translation_mock.rs` fixtures, or (b) using a `#[cfg(test)]` seam on `SessionController` that lets tests force `RunState::Running` with a synthetic id. Call `delete_session` with that id and expect `Err` containing `"active"`. Plan picks the simpler of the two.
- Idempotent missing id: call `delete_session(999_999)`, expect `Ok(())`.

### Manual smoke (added to the existing 10-check gate)
- Hover a past row → trash icon fades in. Hover the live row → no icon.
- Click trash → modal opens, Cancel is default-focused, Esc dismisses cleanly with no DB change.
- Confirm Delete → row vanishes from sidebar; the same text no longer appears in search results.
- While viewing a past transcript, delete it → viewer clears to EmptyState (or to live transcript if recording).
- While a search filter is active, delete a hit → it disappears immediately; clearing the search returns a coherent full list with the deleted row absent.

## 8. Out-of-scope follow-ups

- Multi-select / "delete all in this date group" — defer until a user actually asks.
- Undo via soft-delete + retention window — only if (a) someone reports an accidental delete and (b) the cost of holding orphaned tokens is acceptable.
- Export-as-text before delete — separate feature; orthogonal to deletion.
