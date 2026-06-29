#!/usr/bin/env bash
# Build + publish a signed Android release APK to GitHub Releases, so the in-app
# AndroidUpdateBanner can offer it to friends (it polls the Releases API and
# surfaces the newest release that carries an .apk asset).
#
# Usage:
#   scripts/android-release.sh            # build the CURRENT version, publish
#   scripts/android-release.sh 0.1.8      # bump to 0.1.8 first, then build + publish
#
# Publishes under an `android-vX.Y.Z` tag — deliberately NOT `vX.Y.Z`, so it
# never fires the desktop release CI (.github/workflows/release.yml triggers on
# `v*.*.*` tag pushes and expects a pre-staged macOS draft). The banner reads the
# Releases API and normalises the tag, so the prefix is invisible to friends.
#
# Requires: gh (authenticated), jq, pnpm, cargo, the Android SDK + NDK, and
# src-tauri/gen/android/keystore.properties (the release signing config).
# ponytail: Android toolchain paths are pinned to this machine; override via env.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"
VERSION_ARG="${1:-}"

red()  { printf '\033[31m%s\033[0m\n' "$*"; }
grn()  { printf '\033[32m%s\033[0m\n' "$*"; }
step() { printf '\n\033[1;34m▶ %s\033[0m\n' "$*"; }
die()  { red "error: $*"; exit 1; }

# ── Android build env (pinned; override via the environment) ──────────────────
export ANDROID_HOME="${ANDROID_HOME:-$HOME/Library/Android/sdk}"
export NDK_HOME="${NDK_HOME:-$ANDROID_HOME/ndk/27.3.13750724}"
export JAVA_HOME="${JAVA_HOME:-$(ls -d /opt/homebrew/Cellar/openjdk*/*/libexec/openjdk.jdk/Contents/Home 2>/dev/null | sort -V | tail -1)}"
export PATH="$HOME/.cargo/bin:$ANDROID_HOME/platform-tools:$PATH"

# ── Pre-flight ────────────────────────────────────────────────────────────────
step "Pre-flight"
for c in gh jq pnpm cargo; do command -v "$c" >/dev/null || die "$c not installed / not on PATH"; done
gh auth status >/dev/null 2>&1 || die "gh not authenticated — run 'gh auth login'"
[[ -f src-tauri/gen/android/keystore.properties ]] || die "missing src-tauri/gen/android/keystore.properties (release signing config)"
[[ -d "$NDK_HOME" ]] || die "NDK not found at $NDK_HOME (set NDK_HOME)"
[[ -n "${JAVA_HOME:-}" && -d "$JAVA_HOME" ]] || die "JDK not found (set JAVA_HOME)"
grn "✓ pre-flight ok"

# ── Optional version bump ─────────────────────────────────────────────────────
if [[ -n "$VERSION_ARG" ]]; then
  [[ "$VERSION_ARG" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || die "version '$VERSION_ARG' must be semver (e.g. 0.1.8)"
  CURRENT="$(jq -r .version src-tauri/tauri.conf.json)"
  if [[ "$VERSION_ARG" != "$CURRENT" ]]; then
    step "Bump $CURRENT → $VERSION_ARG  (NOTE: this version is SHARED with the desktop app)"
    # Android only needs tauri.conf.json (it drives versionCode/versionName and
    # getVersion()). package.json is bumped too so the two app-version manifests
    # stay aligned; scripts/release.sh does the full 4-manifest bump for desktop.
    for f in package.json src-tauri/tauri.conf.json; do
      tmp="$(mktemp)"; jq --arg v "$VERSION_ARG" '.version=$v' "$f" >"$tmp" && mv "$tmp" "$f"
    done
    git add package.json src-tauri/tauri.conf.json
    git commit -m "chore(android): bump version to ${VERSION_ARG}"
    grn "✓ bumped + committed"
  fi
fi

VERSION="$(jq -r .version src-tauri/tauri.conf.json)"
TAG="android-v${VERSION}"
step "Publishing Android v${VERSION}  (tag ${TAG})"

# ── Build the signed release APK ──────────────────────────────────────────────
step "Building signed release APK (a few minutes)"
pnpm tauri android build --apk --target aarch64
APK="src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk"
[[ -f "$APK" ]] || die "release APK not found at $APK"
ASSET="$(dirname "$APK")/Voxtide_${VERSION}_android_arm64.apk"
cp -f "$APK" "$ASSET"
grn "✓ built $(basename "$ASSET")"

# ── Publish to GitHub Releases (push first so the tag can point at HEAD) ───────
step "Publishing to GitHub Releases"
git push origin HEAD
if gh release view "$TAG" >/dev/null 2>&1; then
  gh release upload "$TAG" "$ASSET" --clobber
  grn "✓ updated release ${TAG}"
else
  gh release create "$TAG" \
    --target "$(git rev-parse HEAD)" \
    --title "Voxtide Android v${VERSION}" \
    --notes "Android release v${VERSION}. Sideload the APK; it upgrades in place over a prior Voxtide install (same signing key, data preserved)." \
    "$ASSET"
  grn "✓ created release ${TAG}"
fi
gh release view "$TAG" --web 2>/dev/null || gh release view "$TAG"
