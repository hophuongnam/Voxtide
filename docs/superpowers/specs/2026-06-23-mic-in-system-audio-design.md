# Optional microphone capture in System Audio mode

**Date:** 2026-06-23
**Status:** approved, ready to implement

## Goal

In System Audio (Meeting) mode, let the user optionally capture their microphone
*in addition to* system audio, so a meeting where the remote side comes through
the speakers can be transcribed and translated **two-way**: remote voice
(language A, via speakers) → B, and the local user's voice (language B, via mic)
→ A. One bilingual transcript.

Off by default. The toggle persists immediately.

## Key constraint (non-obvious)

`build_initial_config` in `translation/soniox.rs` configures Soniox per mode:

- **Meeting** → `one_way`, `target_language = language_b`, `language_hints = [language_a]`.
- **Conversation** → `two_way` between `language_a` and `language_b`.

A naive blend that left the session `one_way` would make the mic useless — the
user's language-B speech is hinted/tuned for A and never translated. So **mic-on
must flip the session to `two_way`** (the same config Conversation mode already
uses). Rule: use `two_way` when `Conversation || (Meeting && capture_mic)`.

## Decisions (locked with user)

1. **Blend into one stream** — mic + system summed into one 16 kHz mono PCM
   stream, one Soniox session. (Not separate STT sessions — keeps Soniox cost
   flat, reuses the single-stream pipeline.)
2. **Two-way when mic on** — see constraint above.
3. **Toggle in the toolbar** (visible only in System Audio mode), not Settings.
4. **Mic-fails-to-start = fatal mic-permission banner** (reuses the existing
   PermissionBanner). Mic *drops mid-session* = degrade to system-only.
5. **Reuse the existing `default_mic` setting** for device choice — no new picker.

## Components

Every `AudioSource` already emits identical 16 kHz mono `AudioFrame`s
(1600 samples / 100 ms) over an mpsc channel, so mixing is element-wise i16
addition. Six pieces:

### 1. `crates/voxtide-core/src/audio/mix.rs` — `MixSource` (new, the only real logic)

Implements `AudioSource`. Wraps `primary: Box<dyn AudioSource>` (loopback) and
`secondary: Box<dyn AudioSource>` (mic).

`start()`:
- Start `primary` first; on error, propagate as-is (→ `capture-permission`).
- Start `secondary`; on error, return `Error::Audio(format!("microphone: {detail}"))`
  (the `microphone:` prefix is the load-bearing marker for `classify`). Drop the
  already-started primary stream on this path.
- Spawn a mixing task; return its `AudioStream { rx, stop }`.

**Mixing task — system-clock-driven overlay.** The primary's chunk cadence is
the clock. Owns `primary_rx`, `secondary_rx`, both inner `stop` senders, `out_tx`,
and a `mic_buf: VecDeque<i16>`. `tokio::select!` loop:
- output `stop` fired/dropped → drop both inner stops, exit.
- `secondary_rx` recv `Some(s)` → extend `mic_buf` with `s`; if
  `mic_buf.len() > MIC_CAP_SAMPLES` (4800 ≈ 300 ms), drain oldest down to the cap.
  recv `None` → mic EOF: set `mic_open = false`, stop polling this branch
  (guard the branch with `if mic_open`); subsequent output is system-only.
- `primary_rx` recv `Some(frame)` → drain up to 1600 from the front of `mic_buf`,
  `saturating_add` element-wise onto `frame.samples` (system-only beyond what the
  mic buffer holds), send the mixed frame on `out_tx`. recv `None` → primary EOF:
  drop both stops, close `out_tx`, exit.

Notes:
- Draining oldest-first keeps mic in order with bounded delay; the cap only bites
  under sustained clock drift (mic running ahead), where dropping stale mic audio
  is the correct resync. Mic underrun / EOF / never-started all collapse to
  "system-only for that span" — the mic never stalls the primary path.
- Gain: `i16::saturating_add` (sum + clamp). Both are speech-level and rarely
  peak together; STT is robust to mild clipping. `// ponytail: sum+clamp; mic is
  the overlay, system is the clock`.

### 2. `translation/soniox.rs` — `two_way` when mic on

`SessionConfig` gains `capture_mic: bool`. In `build_initial_config`, the Meeting
arm uses the `two_way` block when `cfg.capture_mic`, else the existing `one_way`.
(Equivalently: `one_way` only when `Meeting && !capture_mic`.)

### 3. `config.rs` — persisted flag

`AppConfig.meeting_capture_mic: bool` with `#[serde(default)]` (false), default
`false` in `AppConfig::default()`. Mirror in `src/types.ts`.

### 4. `commands/lifecycle.rs` — StartReq + dispatch + classify

- `StartReq` gains `capture_mic: bool` and `mic_device_id: String`. Mirror in
  `src/lib/ipc.ts`.
- Meeting arm: build the loopback source; if `req.capture_mic`, also build the
  mic source (`MicSource::by_id` when `mic_device_id` non-empty, else
  `default_device()`) and wrap both in `MixSource::new(loopback, mic)`. Set
  `cfg.capture_mic = req.capture_mic`.
- `classify`: keep the `not found`/`no default input device` → `device-missing`
  branch first (so a vanished mic device shows the plain strip). Add: an Audio
  error whose detail contains the `microphone:` marker → `mic-permission`
  (so a mic permission denial routes to the mic banner, not the system-audio one).
  Remaining Audio in Meeting → `capture-permission` as today.

### 5. `src/components/toolbar/Toolbar.svelte` — toggle

A mic toggle shown only when `mode === 'meeting'`, initialized from
`config.meeting_capture_mic`, persisted immediately via the existing
`persist({ meeting_capture_mic })` path. Literal label ("Microphone" /
mic icon), per the literal-labels convention.

### 6. `src/routes/MainApp.svelte` — pass flags on start

`onStart` adds to the `startSession` call:
`capture_mic: mode === 'meeting' && captureMic`,
`mic_device_id: config.config.default_mic ?? ''`.

### 7. `TranscriptPane.svelte` — two-way column headers

**Column placement** is already correct with zero changes: the transcript store
(`stores.svelte.ts`, both the live store and `dbTokensToColumns` replay path)
places by `translation_status` — `original`→left, `translation`→right — and
never branches on `mode`. So a local B→A turn (Soniox emits original=B,
translation=A) lands B-left / A-right, coherent and identical to how
Conversation mode (also two-way) already renders.

Only the column **header hints** branched on `mode === 'meeting'` (e.g.
"VI · target", implying one-way). They now key off a derived
`twoWay = mode === 'conversation' || (mode === 'meeting' && captureMic)`, so a
live mic-blend session shows the bidirectional headers ("EN/VI", "EN ⇄ VI",
"two-way"). `MainApp` passes `captureMic` to the **live** pane only.

## Failure behavior

| Case | Result |
|---|---|
| Mic permission denied / can't open at start | Fatal `mic-permission` → PermissionBanner. User grants access or toggles mic off and retries. |
| Selected mic device missing at start | `device-missing` → plain error strip. |
| Loopback fails at start | `capture-permission` → audio-capture banner (unchanged). |
| Mic drops mid-session (unplug) | Degrade to system-only, no interruption. |

No Info.plist change — `NSMicrophoneUsageDescription` is already present
(Conversation mode uses the mic).

## Testing

- **`mix.rs` unit test** (mock/in-test sources feeding known frames):
  - mic shorter than a system chunk → mixed = system + mic prefix, remainder system-only.
  - mic EOF → later frames pass through equal to system input.
  - sustained mic surplus → `mic_buf` stays ≤ cap (no unbounded growth).
  - saturation: `i16::MAX + positive` clamps, no wrap.
- Existing core/shell tests updated for the new `SessionConfig` / `StartReq` /
  `AppConfig` fields (compiler-driven).
- **Smoke gate** (mandatory before "done"): the standard 10 checks + a new
  mic-blend check — toggle visible only in System Audio mode, persists across
  relaunch, and both directions transcribe **into the correct columns**: the
  remote (A) and the local mic (B) each show original-left / translation-right
  (assert column *correctness*, not merely that rows appear).

## Known limitation

The session row does not persist `capture_mic`, so **replay** of a past
mic-blend session shows the one-way meeting headers (it can't know the session
was two-way). Column *placement* is still correct on replay (status-based).
Persisting the flag for accurate replay headers is deferred — YAGNI until asked.

## Out of scope

Separate STT streams; per-session mic picker; persisting `capture_mic` to the
DB; Info.plist changes (mic key already present).
