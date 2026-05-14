# Versioning

Voxtide follows [Semantic Versioning 2.0.0](https://semver.org).

## Pre-1.0 phase (`0.x.y`)

While the major version stays at `0`, the public surface — storage
format, config schema, IPC, user-visible behavior — is **not stable**.
No deprecation period is promised.

- **MINOR** (`0.X.0`) bumps for: new user-facing features; storage
  format or config-schema changes (e.g. moving secrets from macOS
  Keychain to a JSON file); behavior changes the user will notice
  (e.g. transcript coalesce rule change).
- **PATCH** (`0.x.Y`) bumps for: bug fixes, internal refactors,
  UI polish that doesn't change observable behavior.

## Post-1.0

Standard SemVer:

- **MAJOR** (`X.0.0`) for breaking changes to the public surface.
- **MINOR** (`x.Y.0`) for backwards-compatible new functionality.
- **PATCH** (`x.y.Z`) for backwards-compatible bug fixes.

## Tags

- **Releases:** `vX.Y.Z` (e.g. `v0.1.0`). Pushing this tag triggers
  `.github/workflows/release.yml`, which builds the universal macOS
  `.dmg` plus the x64/arm64 Windows `.msi` installers and attaches
  them to a draft GitHub release.
- **Pre-releases:** SemVer-compliant identifiers — `vX.Y.Z-alpha.1`,
  `vX.Y.Z-beta.1`, `vX.Y.Z-rc.1`.
- **Historical milestone tags:** `v0.1.0-core` and `v0.1.0-ui` exist
  from the Plan 1 / Plan 2 development cycle. They are **not** user
  releases — they marked internal milestones before any installer
  was produced. Going forward, prefer release-line tags (`vX.Y.Z`)
  for anything intended to be installed; do not reuse the trailing-
  label form for milestones to avoid confusion with proper SemVer
  pre-release identifiers.

## Where the version lives

These five must agree at release time:

| File | Field |
|---|---|
| `package.json` | `"version": "X.Y.Z"` |
| `crates/voxtide-core/Cargo.toml` | `version = "X.Y.Z"` |
| `src-tauri/Cargo.toml` | `version = "X.Y.Z"` |
| `src-tauri/tauri.conf.json` | `"version": "X.Y.Z"` |
| `src/types.ts` | `export const VERSION = 'X.Y.Z';` |

A release bump touches all five in one commit titled
`chore(release): vX.Y.Z`, then tags `vX.Y.Z` on that commit:

```sh
git commit -am "chore(release): vX.Y.Z"
git tag -a vX.Y.Z -m "vX.Y.Z"
git push origin main vX.Y.Z   # tag push triggers release.yml
```
