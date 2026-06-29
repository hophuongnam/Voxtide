# Android version — conversation mode only

**Date:** 2026-06-29
**Status:** approved, ready to plan

## Goal

Ship an Android build of Voxtide that does **conversation mode only**: two people,
two languages, talking back-and-forth, the phone lying flat between them. Capture
the **microphone**, stream it to Soniox `two_way`, and render the live bilingual
transcript as a **face-to-face split** (each person reads their own language; the
far person's half is rotated 180°). Distributed as a **signed APK shared with
friends** — no Play Store.

System Audio / Meeting mode does not exist on Android (no system-audio loopback on
the platform, and it isn't wanted here).

## Approach

**Add an Android target to the existing Tauri 2 app — not a rewrite.** Tauri 2
ships native Android support: `tauri android init` wraps the current Svelte
frontend in an Android WebView and compiles `voxtide-core` (mic, Soniox client,
SQLite) for ARM. The Soniox websocket + `two_way` logic stay in Rust untouched, so
the **API key never enters the WebView**.

*Rejected alternative — a mobile web/PWA:* would force reimplementing the Soniox
protocol in JS and would ship the Soniox API key in client-side code where any
friend could extract it. No.

## Decisions (locked with user)

1. **Distribution = signed APK, shared with friends.** One self-generated, **stable
   release keystore** (kept across versions so re-installs upgrade in place). Build
   an **arm64 (`aarch64`) APK**. No Play Store, no AAB, no privacy listing. The
   desktop GitHub `latest.json` updater is **cfg-gated off** on Android; new
   versions = rebuild + re-share the APK.
2. **Layout = face-to-face split.** Portrait-locked. Screen split top/bottom; top
   pane = the far person's language with `transform: rotate(180deg)`, bottom pane =
   the near person's. Each pane scrolls and follows its own tail. Reuses the
   existing column renderers.
3. **Scope = minimal live translator.** API-key setup + language-pair picker +
   face-to-face live view + record start/stop. Sessions still auto-save to SQLite,
   but there is **no in-app history/replay browser in v1** (deferred).

## The load-bearing unknown — audio capture (Phase 0 spike)

Whether `cpal`'s Android input backend works inside a Tauri app is **unproven until
it runs** — it is not assumed to work. A throwaway walking skeleton (mic → Soniox →
one live transcript line on a real device/emulator) picks the path:

- **Path A — reuse `cpal` `MicSource` in Rust** (try first, timeboxed). `cpal` uses
  the `oboe`/AAudio backend on Android and needs a JavaVM/Context from
  `ndk_context`, which the Tauri mobile runtime populates. If input streams work →
  **zero new audio code**; desktop and mobile share one capture→Soniox→DB pipeline.
- **Path B — WebView capture, bridged into the existing pipeline** (fallback, known
  to work). The WebView captures via `navigator.mediaDevices.getUserMedia` +
  an `AudioWorklet`, posts PCM frames over a `tauri::ipc::Channel` to Rust, where a
  **new `WebViewMicSource`** (implements the existing `AudioSource` trait) feeds
  them through the **existing resample+chunk helpers** (`audio/cpal_pipeline.rs`)
  and into the unchanged Soniox client. JS ships raw i16 PCM at the WebView's native
  sample rate + the rate; Rust does the resample to 16 kHz mono — keeps the tricky
  DSP in tested Rust. **Soniox stays in Rust either way; never reimplement the
  protocol in JS.**

The `AudioSource`/`MicSource` abstraction already exists, so Path B diverges only at
the capture leg — everything downstream is shared.

## Architecture changes (by area)

### 1. Build target & toolchain (Phase 0)

- Install: Rust Android targets (`aarch64-linux-android` for devices; add
  `x86_64-linux-android` for the emulator), Android **NDK** (via SDK Manager), and
  set `ANDROID_HOME` / `NDK_HOME`. (Host already has `tauri-cli 2.11.1`, the SDK at
  `~/Library/Android/sdk`, and `adb`; env vars are unset, targets/NDK are not yet
  installed.)
- `tauri android init` → generates `src-tauri/gen/android` (Gradle project).

### 2. Mobile entry point (Phase 0)

Tauri mobile requires the app in a library with a mobile entry function. Refactor
`src-tauri/src/main.rs`:
- Move the `tauri::Builder` chain into `pub fn run()` in a new `src-tauri/src/lib.rs`,
  annotated `#[cfg_attr(mobile, tauri::mobile_entry_point)]`.
- `main.rs` shrinks to `fn main() { app_lib::run() }`.
- Add a `[lib]` target to `src-tauri/Cargo.toml` with
  `crate-type = ["staticlib", "cdylib", "rlib"]` and a `name`.

### 3. cfg-gate desktop-only code so Android compiles (Phase 0)

Wrap with `#[cfg(desktop)]` (or `#[cfg(not(target_os = "android"))]`) in `lib.rs`
and friends:
- `tauri_plugin_global_shortcut` (`main.rs:34`) + `hotkey.rs` registration.
- `tauri_plugin_updater` (`main.rs:36`).
- The **overlay**: the `overlay` window (`tauri.conf.json` `windows[1]`), the
  capability denylist (`main.rs:43`), and the three overlay commands
  (`commands/overlay.rs`, registered `main.rs:120-122`).
- Single-instance guard (`main.rs:24` context) — desktop-only.
- The `macos-private-api` feature + macOS dock/reopen/CloseRequested handlers.
- Android uses only the `main` window; exclude `overlay` via a platform config
  override or runtime-gated window creation (confirm the exact mechanism in Phase 0).

### 4. Data-dir injection (Phase 1) — small, shell-only

`voxtide-core` is already clean: `Store::open(&Path)` and `Keychain::new(path)`
both take an injected path. The **only** offender is `src-tauri/src/state.rs:20`,
whose free `data_dir()` calls `dirs::data_dir()` — wrong/None on Android. Fix:
resolve the base dir from Tauri's path API (`app.path().app_data_dir()`), which is
correct on every platform, and pass it into `init()` instead of calling `dirs`.
Desktop behavior is unchanged (same Application Support dir). **No `voxtide-core`
change.** Secrets stay as the existing 0600 JSON — already private in Android's app
sandbox, no Android Keystore needed.

### 5. Microphone permission (Phase 0)

- Add `<uses-permission android:name="android.permission.RECORD_AUDIO"/>` to the
  generated `AndroidManifest.xml`.
- Request the runtime permission before capture (Android 6+). Small native glue in
  the generated Android project (Kotlin `requestPermissions`, or a minimal plugin).
- **Path B also** needs the Android WebView's mic permission granted — wire
  `onPermissionRequest` (wry may not do this for free). Verify in the Phase 0 spike.

## Mobile UI — face-to-face split (Phase 2)

Render platform-conditionally: on Android, mount a new `FaceToFaceView` instead of
the desktop `MainApp` chrome (detect via Tauri `platform()` / a viewport check).

- **The split view** reuses the existing `Line` / `SpeakerChip` renderers and the
  transcript store's data, but **buckets lines by `language`, not by
  `translation_status`**: pane A = all lines where `language === language_a`, pane
  B = `language_b`, so each person reads only their own language. (The desktop view
  buckets by status — original-left/translation-right — which *mixes* languages per
  column when speakers alternate; correct for one reader, wrong for face-to-face.)
  A small pure `bucketByLanguage(lines, aCode, bCode)` does the split. **Risk: this
  is raw string-equality between Soniox's emitted `language` tag and the stored
  config code — if they differ (region suffix / casing), both panes go silently
  empty. Confirm the real tags match in Phase 0, and test the no-match case.** Mobile
  lays the two language panes stacked:
  - **Top pane** = far person's language (`language_a` or `language_b` per the swap
    state), wrapped in `transform: rotate(180deg)`. Rotating a scroll container
    flips visual scroll direction — keep logical "scroll to newest" and let the
    transform handle visuals; verify follow-tail in both panes during the build.
  - **Bottom pane** = near person's language.
  - Each pane follows its own tail; live partial tokens render as they stream.
- **Center divider** holds: record start/stop (reuses `start_session`/`stop_session`
  with `mode = Conversation`, `language_a`/`language_b`, default mic
  `device_id = ""`), a language-swap button, and a connection/latency dot.
- **First-run setup screen** (reached via a gear in the divider): API-key field
  (reuses `set_api_key`) + language-pair picker (reuses `LangPicker`/`LangPair`).
  Required before record can start.
- **Portrait-locked.**

Literal UI labels throughout (per the project's literal-labels convention).

## Distribution (Phase 3)

- Generate one release keystore (`keytool`), store it safely, and configure Gradle
  signing in `gen/android` to use it (not the throwaway debug key) so updates
  install over old versions.
- Build a **signed arm64 APK**; install via `adb install` / share the file. Add
  32-bit `armv7` only if a friend has an ancient device.
- Updater stays gated off (no updater artifacts on Android).

## Sequencing

- **Phase 0 (riskiest, first):** toolchain + targets + NDK, `tauri android init`,
  mobile entry refactor, cfg-gate desktop-only code to compile, `RECORD_AUDIO` +
  runtime permission, **audio walking skeleton → decide Path A vs B.**
- **Phase 1:** data-dir injection; verify SQLite + secrets read/write on-device.
- **Phase 2:** `FaceToFaceView` + setup screen + record wiring + portrait lock.
- **Phase 3:** release keystore, Gradle signing, signed APK, on-device smoke.

## Mobile smoke gate (mandatory before "done")

On a real device: boot → grant mic permission → enter API key → pick language pair
→ start record → speak language A and language B alternately → **live tokens appear
in BOTH panes**, each in its own language → speaker A/B diarization chips correct →
**top pane reads right-side-up when viewed from across the table** (rotation
correct) and each pane follows its tail → stop → relaunch → API key + language pair
persisted, and the session was saved to the DB. (Same spirit as the desktop 10-check
smoke gate, adapted to mobile + the face-to-face rotation.)

## Out of scope (v1)

System Audio / Meeting mode, loopback (macOS/Windows), MixSource; overlay window,
global hotkey, dock/tray; auto-updater; in-app session history/replay browser
(sessions still save); background / screen-off capture (would need an Android
foreground Service); Play Store / AAB / privacy listing; iOS.

## Open risks

- **Path A viability** (cpal Android input in Tauri) — resolved by the Phase 0
  spike; Path B is the de-risked fallback.
- **Language-tag match** (the silent-empty-panes ship-breaker) — Soniox's emitted
  `language` tag must string-equal the stored config code for bucketing to fill the
  panes; observed against real output in Phase 0, normalized if needed.
- **Rotated-pane follow-tail** — scroll direction under a 180° transform; verified
  during Phase 2.
- **WebView mic permission** (Path B only) — `onPermissionRequest` wiring on Android.
