#!/usr/bin/env bash
# Release Voxtide: bump 4 manifests, build signed updater artifacts, publish to GitHub.
#
# Usage:
#   TAURI_SIGNING_PRIVATE_KEY_PASSWORD='...' scripts/release.sh <version> [notes-file]
#
# Example:
#   scripts/release.sh 0.1.2 RELEASE_NOTES.md
#   scripts/release.sh 0.1.2          # notes default to "Release vX.Y.Z"
#
# Requires: gh (authenticated), jq, pnpm, cargo, ~/.tauri/voxtide.key,
# and TAURI_SIGNING_PRIVATE_KEY_PASSWORD in the environment.

set -euo pipefail

VERSION="${1:-}"
NOTES_FILE="${2:-}"
KEY_PATH="${HOME}/.tauri/voxtide.key"
REPO_URL="https://github.com/hophuongnam/Voxtide"

red()   { printf '\033[31m%s\033[0m\n' "$*"; }
green() { printf '\033[32m%s\033[0m\n' "$*"; }
blue()  { printf '\033[34m%s\033[0m\n' "$*"; }
step()  { printf '\n\033[1;34m▶ %s\033[0m\n' "$*"; }
die()   { red "error: $*"; exit 1; }

[[ -n "$VERSION" ]] || die "missing <version> arg (e.g. 0.1.2)"
[[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[A-Za-z0-9.]+)?$ ]] || die "version '$VERSION' must be semver (e.g. 0.1.2 or 0.1.2-rc1)"

cd "$(git rev-parse --show-toplevel)"

# ── Pre-flight ────────────────────────────────────────────────────────────────
step "Pre-flight checks"

[[ "$(git rev-parse --abbrev-ref HEAD)" == "main" ]] || die "not on main branch"
[[ -z "$(git status --porcelain)" ]] || die "working tree not clean — commit or stash first"
[[ -f "$KEY_PATH" ]] || die "signing key not found at $KEY_PATH (regenerate with 'npx tauri signer generate -w $KEY_PATH')"
[[ -n "${TAURI_SIGNING_PRIVATE_KEY_PASSWORD:-}" ]] || die "TAURI_SIGNING_PRIVATE_KEY_PASSWORD not set in environment"
command -v gh >/dev/null  || die "gh not installed"
command -v jq >/dev/null  || die "jq not installed"
command -v pnpm >/dev/null || die "pnpm not installed"
gh auth status >/dev/null 2>&1 || die "gh not authenticated — run 'gh auth login'"

if git rev-parse "v${VERSION}" >/dev/null 2>&1; then
  die "tag v${VERSION} already exists locally"
fi
if gh release view "v${VERSION}" >/dev/null 2>&1; then
  die "GitHub release v${VERSION} already exists"
fi

# Detach any stale Voxtide DMG mount (per memory: Tauri's bundle_dmg.sh fails silently otherwise).
if hdiutil info | grep -q -i voxtide; then
  blue "detaching stale Voxtide DMG mount(s)…"
  while IFS= read -r dev; do
    hdiutil detach -force "$dev" || true
  done < <(hdiutil info | awk '/^\/dev\// { dev=$1 } /Voxtide/ { print dev }' | sort -u)
fi

green "✓ pre-flight ok"

# ── Bump version in 4 manifests + refresh Cargo.lock ──────────────────────────
step "Bumping version → ${VERSION}"

# package.json (frontend manifest, also feeds Vite __APP_VERSION__)
tmp="$(mktemp)"
jq --arg v "$VERSION" '.version = $v' package.json > "$tmp" && mv "$tmp" package.json

# tauri.conf.json
tmp="$(mktemp)"
jq --arg v "$VERSION" '.version = $v' src-tauri/tauri.conf.json > "$tmp" && mv "$tmp" src-tauri/tauri.conf.json

# Cargo.toml (src-tauri) — anchored pattern so we only hit [package].version
sed -i '' -E "s/^version = \"[0-9]+\.[0-9]+\.[0-9]+([-A-Za-z0-9.]*)?\"$/version = \"${VERSION}\"/" src-tauri/Cargo.toml
grep -q "^version = \"${VERSION}\"$" src-tauri/Cargo.toml || die "version bump failed in src-tauri/Cargo.toml"

# Cargo.toml (voxtide-core)
sed -i '' -E "s/^version = \"[0-9]+\.[0-9]+\.[0-9]+([-A-Za-z0-9.]*)?\"$/version = \"${VERSION}\"/" crates/voxtide-core/Cargo.toml
grep -q "^version = \"${VERSION}\"$" crates/voxtide-core/Cargo.toml || die "version bump failed in crates/voxtide-core/Cargo.toml"

blue "refreshing Cargo.lock…"
cargo check -p voxtide -p voxtide-core --quiet

git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml crates/voxtide-core/Cargo.toml Cargo.lock
git commit -m "chore: bump version to ${VERSION}"
git tag "v${VERSION}"
green "✓ committed + tagged v${VERSION}"

# ── Build with signing env vars ───────────────────────────────────────────────
step "Building signed bundle (this takes a few minutes)"

export TAURI_SIGNING_PRIVATE_KEY="$(cat "$KEY_PATH")"
# Password already in environment per pre-flight check.

pnpm tauri build

DMG="target/release/bundle/dmg/Voxtide_${VERSION}_aarch64.dmg"
TARBALL="target/release/bundle/macos/Voxtide.app.tar.gz"
SIG="${TARBALL}.sig"

[[ -f "$DMG" ]]     || die "expected DMG not found at $DMG"
[[ -f "$TARBALL" ]] || die "expected updater tarball not found at $TARBALL — is createUpdaterArtifacts set?"
[[ -f "$SIG" ]]     || die "expected signature not found at $SIG — did signing actually run?"

green "✓ artifacts produced:"
ls -lh "$DMG" "$TARBALL" "$SIG" | awk '{print "    " $5 "  " $NF}'

# ── Generate latest.json (updater manifest) ───────────────────────────────────
step "Generating latest.json"

PUB_DATE="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
SIGNATURE="$(cat "$SIG")"

if [[ -n "$NOTES_FILE" && -f "$NOTES_FILE" ]]; then
  NOTES="$(cat "$NOTES_FILE")"
else
  NOTES="Release v${VERSION}"
fi

LATEST_JSON="target/release/bundle/macos/latest.json"
jq -n \
  --arg version "$VERSION" \
  --arg notes "$NOTES" \
  --arg pub_date "$PUB_DATE" \
  --arg signature "$SIGNATURE" \
  --arg url "${REPO_URL}/releases/download/v${VERSION}/Voxtide.app.tar.gz" \
  '{
     version: $version,
     notes: $notes,
     pub_date: $pub_date,
     platforms: {
       "darwin-aarch64": { signature: $signature, url: $url }
     }
   }' > "$LATEST_JSON"

green "✓ wrote $LATEST_JSON"

# ── Confirm before pushing to remote ──────────────────────────────────────────
step "Ready to publish"
cat <<EOF
Local state:
  • commit: $(git log -1 --oneline)
  • tag:    v${VERSION}
  • bundle: $DMG
            $TARBALL
            $SIG
            $LATEST_JSON

About to:
  1. git push origin main v${VERSION}
  2. gh release create v${VERSION} --title "v${VERSION}" --notes-file <notes>
     attaching: DMG, .tar.gz, .sig, latest.json

EOF
read -r -p "Proceed? [y/N] " ans
[[ "$ans" == "y" || "$ans" == "Y" ]] || die "aborted by user (local commit + tag preserved)"

# ── Push + release ────────────────────────────────────────────────────────────
step "Pushing to origin"
git push origin main "v${VERSION}"

step "Creating GitHub release"
NOTES_TMP="$(mktemp)"
printf '%s' "$NOTES" > "$NOTES_TMP"
gh release create "v${VERSION}" \
  --title "v${VERSION}" \
  --notes-file "$NOTES_TMP" \
  "$DMG" "$TARBALL" "$SIG" "$LATEST_JSON"
rm -f "$NOTES_TMP"

green ""
green "✓ released v${VERSION}"
gh release view "v${VERSION}" --web 2>/dev/null || gh release view "v${VERSION}"
