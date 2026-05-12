# Voxtide — Design Specification

**Status:** Approved (v1)
**Date:** 2026-05-12
**Author:** Nam Ho

## 1. Overview

Voxtide is a cross-platform (macOS, Windows) desktop application that captures live audio and renders real-time speech translation using Soniox's `stt-rt-v4` real-time API. It targets two scenarios with a single click-to-switch mode:

- **Meeting mode** — translate the remote speaker on Zoom / Meet / Teams / YouTube / etc. by capturing system audio loopback.
- **Conversation mode** — translate face-to-face conversations between two languages on a single device, by capturing the microphone with Soniox `two_way` translation.

Each mode uses a single Soniox WebSocket session. The mic is ignored in Meeting mode; system audio is ignored in Conversation mode. This keeps cost predictable (one streaming connection at a time) and the audio routing simple.

## 2. Goals & non-goals

### Goals
- Sub-second latency from spoken word to on-screen translation.
- Stable on macOS 14.4+ and Windows 10/11 (x64 + arm64).
- Single executable, no virtual audio device required on either platform.
- Local-only by default. API key stored in OS keychain. No telemetry.
- Searchable history of past sessions.

### Non-goals (v1)
- Linux support.
- Translating the user's own voice during Meeting mode.
- Custom vocabulary / context biasing.
- Cloud sync of transcripts.
- Multi-party diarization in Conversation mode (single-mic two-voice; can enable Soniox diarization later).

## 3. Modes & translation configuration

| Mode | Audio source | Soniox `translation` block | Other flags |
|---|---|---|---|
| **Meeting** | System audio loopback only | `{ "type": "one_way", "target_language": "<user-lang>" }` | `language_hints: ["<picked-source>"]` (no auto-detect) |
| **Conversation** | Microphone only | `{ "type": "two_way", "language_a": "<A>", "language_b": "<B>" }` | `enable_speaker_diarization: true` |

The user picks two languages (A, B) in the top bar. In settings the user marks one of A/B as "my language" — this becomes `target_language` in Meeting mode. In Conversation mode both languages are sent to Soniox as a two-way pair regardless of which is "mine".

Source language in Meeting mode is **locked to the picked non-"mine" language** (sent as `language_hints`); we don't enable Soniox `enable_language_identification`. The user explicitly picks the language they are listening to.

Speaker diarization is **on in Conversation mode** so the UI can render colored speaker chips (A/B) per turn; mapped from Soniox `speaker: "1"`/`"2"`. Diarization is **off in Meeting mode** (single remote source).

Language defaults (A, B, and which is "mine") are persisted to a local config file and restored on next launch.

## 4. Architecture

```
┌─────────────────────────────────────────────┐
│ Tauri 2 application                         │
│                                             │
│  ┌──────────────┐      ┌──────────────────┐ │
│  │  Frontend    │◀────▶│  Rust core       │ │
│  │  Svelte 5    │ IPC  │  (src-tauri/)    │ │
│  │  + Tailwind  │      │                  │ │
│  └──────────────┘      │  ▸ audio capture │ │
│   ▲       ▲            │  ▸ resampler     │ │
│   │       │            │  ▸ Soniox WS     │ │
│   │       │            │  ▸ SQLite store  │ │
│  Main   Overlay        │  ▸ keychain      │ │
│ window  window         └──────────────────┘ │
└─────────────────────────────────────────────┘
            │
            ▼
   wss://stt-rt.soniox.com/transcribe-websocket
```

### 4.1 Tech stack

- **Tauri 2** desktop shell. Two windows: main + overlay.
- **Rust** backend in `src-tauri/`.
- **Svelte 5 + TypeScript + Vite + Tailwind** frontend.
- **SQLite** via `sqlx` for persistence (bundled, no server).
- **`keyring` crate** for OS keychain (Keychain on macOS, Credential Manager on Windows).
- **`tokio-tungstenite`** for the Soniox WebSocket client.

### 4.2 Translation provider abstraction

```rust
trait TranslationProvider {
    async fn open(&mut self, cfg: SessionConfig) -> Result<()>;
    async fn send_audio(&mut self, pcm: &[u8]) -> Result<()>;
    async fn next_event(&mut self) -> Result<TranslationEvent>;
    async fn close(self) -> Result<()>;
}
```

v1 ships one implementation: `SonioxBYOK`. A future `ManagedServer` impl (server proxies WebSocket using a master key) can be added without changing call sites.

## 5. Audio capture layer

Output is always normalized to **16 kHz mono signed-16-bit-LE PCM**, in **100 ms chunks**, before being sent as binary WebSocket frames.

| Platform | Microphone | System (loopback) |
|---|---|---|
| **macOS** | `cpal` (AVAudioEngine under the hood) | Core Audio process taps (`AudioHardwareCreateProcessTap` + aggregate device + `AudioDeviceCreateIOProcIDWithBlock`), via `coreaudio-sys` FFI. Requires macOS 14.4+. Fallback to ScreenCaptureKit on 13.0–14.3. |
| **Windows** | `cpal` | `cpal` WASAPI loopback on the default render endpoint (no extra deps). |

A platform abstraction `trait AudioSource { fn start(...) -> Box<dyn Stream>; }` hides the OS specifics. Each platform crate is gated by `#[cfg(target_os = "...")]`.

The resampler uses `rubato` (high-quality SincFixedIn) when the device sample rate isn't already 16 kHz.

## 6. Soniox protocol details

**Endpoint:** `wss://stt-rt.soniox.com/transcribe-websocket`

**Initial config message** (sent immediately after WS open):

```json
{
  "api_key": "<from keychain>",
  "model": "stt-rt-v4",
  "audio_format": "pcm_s16le",
  "sample_rate": 16000,
  "num_channels": 1,
  "enable_endpoint_detection": true,
  "enable_speaker_diarization": true,
  "language_hints": ["en"],
  "translation": {
    "type": "two_way",
    "language_a": "en",
    "language_b": "vi"
  }
}
```

**Audio stream:** raw bytes as binary WebSocket frames.
**End of audio:** send empty string `""`. Server responds with `{"finished": true}`.

**Response tokens** are JSON messages of the form:

```json
{
  "tokens": [
    { "text": "Hello", "is_final": true,  "language": "en", "translation_status": "original"    },
    { "text": "Xin chào", "is_final": true, "language": "vi", "translation_status": "translation", "source_language": "en" }
  ],
  "final_audio_proc_ms": 4800,
  "total_audio_proc_ms": 5250,
  "finished": false
}
```

Routing:
- `translation_status == "original"` → source column / source side of overlay
- `translation_status == "translation"` → translated column / overlay text
- `is_final == false` → live ghost text (replace each tick, do not commit)
- `is_final == true` → commit to UI + persist to SQLite

### 6.1 Reconnect policy

On WebSocket error or close-before-finish: exponential backoff (250 ms → 500 → 1000 → 2000, cap 5 s, max 6 attempts), open a fresh session with the same config, and discard in-flight non-final tokens (they would otherwise duplicate). The user sees a small "reconnecting…" badge in the toolbar.

## 7. UI

### 7.1 Main window (~900×600, resizable)

- **Top bar:** Mode toggle (Meeting / Conversation) • Language A picker • Language B picker • Audio source picker (mode-aware: mic devices in Conversation, system audio sources in Meeting) • Start/Stop • Overlay toggle • Settings.
- **Body:** two-column live transcript (Original | Translation), auto-scroll, sticky-to-bottom. Non-final tokens render as a lighter "ghost" line with a blinking block caret that replaces in place. Each line: timestamp (mono, dim) + text; in Conversation mode each line is prefixed with a colored A/B speaker chip and language code.
- **Left sidebar (240 px):** search box (⌘K) + session history. Each item shows date/time, duration, language pair with arrow, mode tag (uppercase), and a 2-line preview; active session shows a small REC dot.
- **Status bar:** see §7.4 fields.

### 7.2 Overlay window (frameless, always-on-top, draggable)

- Default: **600 × ~190 px** (5-line variant); also a compact **600 × ~110 px** (3-line) variant the user can toggle.
- Bottom-center of the primary display by default; remembers last position per display.
- Renders a **5-line rolling buffer** of translated text. Newest line is largest (17 px, semibold) and shows the streaming caret; older lines progressively smaller (14 px) and fade toward the top (opacity ramped from ~0.35 oldest → 1.0 newest).
- **Click-through model:** the entire window is click-through by default (`ignoresMouseEvents` / `WS_EX_TRANSPARENT`). When the cursor enters the window's bounding rect, a thin control strip (~24 px tall) reveals on the top edge with: state dot + state label (`EN → VI` / `RECONNECTING` / `IDLE`), center drag region, and close button. While the strip is visible, the window briefly captures mouse events so the user can drag or close. On `mouseLeave` (or after ~1.5 s of no movement over the strip), it auto-hides and the window returns to fully click-through. No always-visible buttons.
- States: `active`, `reconnecting` (shows backoff attempt number and retry-in-N s), `idle` (shows hotkey hint).
- Synced with the main window via the Tauri event bus (Rust core emits a single `transcript:update` event consumed by both windows).

### 7.3 Settings sheet (560 × 680)

Sections, top to bottom:
1. **Soniox API key** — masked field, "Last updated" timestamp, "Replace" button. Paste-once, stored in OS keychain.
2. **Default languages** — two side-by-side `LangCard` components (code + name + "Language A/B"); click one to mark it `MY LANGUAGE` (drives `target_language` in Meeting mode).
3. **Default audio source per mode** — one row for Meeting (system output picker), one for Conversation (microphone picker).
4. **Global hotkey** — `Ctrl + Shift + V` default.
5. **Appearance** — segmented control: Light / Dark / System.

### 7.4 Visual design

Prototype reference lives in the repo at `design/v1/` — read `design/v1/chat-transcript.md` for design intent, then the JSX files for exact tokens, spacing, and component composition.

- **Typography:** Geist + Geist Mono (Google Fonts).
- **Palette:** dark cool-neutral oklch (`bg: oklch(0.16 0.005 250)` … `surface3: oklch(0.28 0.008 250)`); single cyan accent `oklch(0.80 0.13 205)`; REC `oklch(0.72 0.20 25)`; warn `oklch(0.82 0.14 80)`; ok `oklch(0.78 0.13 155)`.
- **Main window:** 920 × 600. Toolbar (48 px) • content row (sidebar 240 px + transcript) • status bar (28 px, mono 10.5 px).
- **Status bar fields** (with hide-priority for narrow widths, dropped right-to-left as space shrinks): REC dot + elapsed • level meter (14 bars) + dB • Soniox model (`SONIOX · stt-rt-v4`) • latency ms or `ws idle` • translation block summary (`one_way → VI` / `two_way · EN ⇄ JA`) • audio format (`16 kHz · mono · s16le`).
- **Latency** is a **client-derived metric**: median ms between an audio chunk send and the first `is_final: true` token covering that chunk's time range. Not a Soniox API field.
- **Lang chip badge in toolbar** uses the label `MY LANGUAGE` (matches the settings sheet), not `YOU` — single consistent term across the app.
- **Hotkey:** `Ctrl + Shift + V` on both platforms (no platform-specific binding).
- **App icon basis:** cyan rounded-square with white sound-wave glyph (see `voxtide.jsx` `Toolbar` wordmark).

## 8. Data model (SQLite)

```sql
CREATE TABLE sessions (
  id           INTEGER PRIMARY KEY,
  started_at   INTEGER NOT NULL,   -- unix epoch ms
  ended_at     INTEGER,
  mode         TEXT NOT NULL,      -- 'meeting' | 'conversation'
  lang_a       TEXT NOT NULL,
  lang_b       TEXT NOT NULL,
  device_label TEXT,
  duration_ms  INTEGER
);

CREATE TABLE tokens (
  id         INTEGER PRIMARY KEY,
  session_id INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
  ts_ms      INTEGER NOT NULL,     -- arrival time, ms from session start
  text       TEXT NOT NULL,
  language   TEXT,
  status     TEXT NOT NULL,        -- 'original' | 'translation'
  speaker    TEXT
);

CREATE INDEX idx_tokens_session ON tokens(session_id, ts_ms);

CREATE VIRTUAL TABLE tokens_fts USING fts5(
  text,
  content='tokens',
  content_rowid='id'
);
-- triggers keep tokens_fts in sync with tokens (insert/delete only;
-- non-final tokens are never written so no update path needed)
```

Only `is_final: true` tokens are persisted. Non-final ghost text lives only in memory.

## 9. Error handling

| Condition | Behavior |
|---|---|
| No API key set | Settings sheet opens on launch; Start button disabled. |
| Soniox auth failure | Toast with server's error string; open settings. |
| macOS mic permission denied | In-app explainer panel with deep link to `x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone`. |
| macOS Core Audio tap not granted | Same pattern, deep link to `Privacy_Audio`. |
| Network drop | Reconnect with backoff (§6.1); show "reconnecting…" badge. |
| Audio device unplugged | Stop session, surface "device disconnected" toast, keep history intact. |
| Soniox rate limit / quota | Surface server message verbatim; Start re-enabled after user acknowledges. |

## 10. Build & test

- **CI:** GitHub Actions matrix.
  - macOS arm64 + x86_64 universal `.dmg` (signed + notarized later — out of v1 ship-blocker, but bundle structure ready).
  - Windows x64 + arm64 `.msi`.
- **Rust unit tests:** resampler accuracy, Soniox message parser, SQLite migrations, keychain wrapper.
- **Integration test:** feed a known WAV file through the platform-agnostic audio pipeline + a `MockTranslationProvider` → assert exact token timeline.
- **Manual smoke checklist** per platform: mic capture, system loopback, two languages, overlay always-on-top behavior, hotkey, history search.

## 11. Out of scope (v1)

- Linux.
- Translating the user's own voice during Meeting mode.
- Custom vocabulary / context biasing (Soniox supports it via `context` field; trivial follow-up).
- Cloud sync.
- Code-signing & notarization workflows.
- Light theme (segmented control ships but only dark theme rendered; light theme is a follow-up).
- Sidebar history grouping / virtualization at >100 sessions (flat list ships in v1).

## 12. Open questions for follow-up

- Soniox session duration / per-key rate limits — to confirm when implementing reconnect logic.
- Exact macOS version split between Core Audio taps and ScreenCaptureKit fallback (some 14.x point releases shipped tap fixes).
- Whether to expose Soniox `enable_endpoint_detection` as a user setting or keep it hard-on.
- App icon system: 1024 marketing icon, 32 / 16 list icons, and macOS menu-bar monochrome variant — design the 16 px legibility before commissioning the full set.
