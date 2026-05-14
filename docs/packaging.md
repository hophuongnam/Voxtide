# Voxtide packaging

## One-time setup
- macOS: `xcode-select --install`
- Windows: install VS 2022 Build Tools with the "Desktop development with C++" workload
- `pnpm install --frozen-lockfile`

## Generate icons
```bash
pnpm icon:master      # SVG → 1024 PNG + menu-bar template
pnpm icon:build       # PNG → .icns / .ico / 32/128/128@2x
```
The 1024 master and derived assets are committed; rerun the scripts only when the icon design changes.

## Build a local installer
- macOS (universal): `pnpm tauri build --target universal-apple-darwin`
- Windows x64: `pnpm tauri build`
- Output: `src-tauri/target/<target>/release/bundle/`

## Code signing
v1 ships unsigned per the spec (`docs/superpowers/specs/2026-05-12-voxtide-design.md` §11). When signing is added, populate the following GitHub Actions secrets:
- `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID`
- `WINDOWS_CERTIFICATE`, `WINDOWS_CERTIFICATE_PASSWORD`

## Manual smoke checklist (per platform, before tagging a release)
1. Settings sheet: paste Soniox key, switch theme through Light/Dark/System, change languages.
2. Meeting mode + system source: a video on YouTube produces continuous translations.
3. Conversation mode + mic: two voices on a single mic produce A/B chips.
4. Overlay window: open via toolbar; default state is click-through; hover shows the 24 px strip; close X dismisses; the window returns to click-through ~1.5 s after hover ends.
5. Global hotkey: Ctrl+Shift+V from another app toggles a session.
6. Resize main window to 480 px, 600 px, 720 px, 920 px — status bar drops fields per the hide-priority.
7. History: at least one session shows under `Today`; search returns FTS5 hits.
8. Quit + relaunch: settings restore, history persists.
