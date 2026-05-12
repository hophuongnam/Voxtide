# Voxtide — UI design brief (v2 revision)

Paste this whole brief into a fresh Claude Design thread. The previous bundle (`design/v1/`) was a good first pass; this brief restates every constraint so the designer doesn't need to remember it. Build artboards on a single design canvas, side-by-side, as before.

---

## 1. App context

**Voxtide** is a cross-platform (macOS, Windows) desktop translation app built in Tauri 2 + Rust + Svelte 5. It captures audio and shows real-time speech translation via Soniox's `stt-rt-v4` real-time WebSocket API. Two modes, one click to switch:

- **Meeting** — captures **system audio loopback** only (e.g. Zoom / Meet / YouTube). One-way translation from picked source → user's language. **Speaker diarization is on** — multiple remote participants are differentiated.
- **Conversation** — captures **microphone** only. Two-way translation between language A ↔ language B on a single device (face-to-face). **Speaker diarization is on** — two voices via one mic, split into chips A / B.

The app sits next to Zoom and similar tools — visual aesthetic should not compete with meeting UI.

## 2. Design system (preserve from v1 unchanged)

### Typography
Geist (UI) + Geist Mono (numerics, codes, timestamps) — Google Fonts.

### Dark palette (oklch — keep exactly)
| Token | Value |
|---|---|
| `bg` | `oklch(0.16 0.005 250)` |
| `bgDeep` | `oklch(0.12 0.005 250)` |
| `surface` | `oklch(0.20 0.006 250)` |
| `surface2` | `oklch(0.24 0.007 250)` |
| `surface3` | `oklch(0.28 0.008 250)` |
| `border` | `oklch(0.30 0.008 250)` |
| `borderHi` | `oklch(0.40 0.010 250)` |
| `text` | `oklch(0.97 0.003 250)` |
| `muted` | `oklch(0.70 0.005 250)` |
| `subtle` | `oklch(0.52 0.005 250)` |
| `dim` | `oklch(0.38 0.005 250)` |
| `accent` (cyan) | `oklch(0.80 0.13 205)` |
| `accentDim` | `oklch(0.42 0.08 205)` |
| `accentInk` | `oklch(0.18 0.04 205)` |
| `rec` (red) | `oklch(0.72 0.20 25)` |
| `warn` (amber) | `oklch(0.82 0.14 80)` |
| `ok` (green) | `oklch(0.78 0.13 155)` |

### Light palette (design from scratch — see §3 item 2 below)
Mirror the dark palette's chroma/hue but invert lightness for the neutrals. Cyan accent stays. Spec it explicitly in the v2 deliverable.

### Window dimensions
- Main window: **920 × 600**, resizable. Toolbar 48 px, status bar 28 px, sidebar 240 px wide.
- Overlay (5-line): **600 × ~190 px**, frameless, always-on-top.
- Overlay (3-line compact): **600 × ~110 px**.
- Settings sheet: **560 × 680**, frameless modal.

### Toolbar contents (left to right)
Traffic lights · vertical divider · Voxtide wordmark + glyph · vertical divider · Mode toggle (Meeting / Conversation) · Lang pair (chip A · swap arrow · chip B; the "mine" chip gets a tiny corner badge labelled `YOU` — keep this label on the chip; the settings card uses `MY LANGUAGE` because it's a card not a chip) · Audio source picker (mode-aware: system source in Meeting, mic device in Conversation) · flex spacer · Overlay-toggle icon button · Settings icon button · Primary Start/Stop button (cyan when idle, red when recording).

### Status bar fields (mono 10.5 px, dim, with separators)
REC dot + state (REC / IDLE) + elapsed `hh:mm:ss` · level meter (14 vertical bars) + dB · `SONIOX · stt-rt-v4` · latency `262 ms` or `ws idle` · translation block summary (`one_way → VI` / `two_way · EN ⇄ JA`) · audio format (`16 kHz · mono · s16le`).

### Transcript line composition
Timestamp (mono, dim, 52 px column) + speaker chip (colored, 16×16 rounded square with letter) + speaker language code (mono, subtle) + text. Live (non-final) text is muted-color with a blinking block caret at the end. Final text is full-contrast.

## 3. Revisions for v2

### 1 · Speaker chips in **both** modes
v1 rendered speaker chips on Conversation lines only. **In v2, every transcript line in both modes carries a speaker chip + language code**, including Meeting (where multiple remote participants on a call are visually separated). Chips cycle through accent variants: A = cyan accent, B = warm red `oklch(0.78 0.13 25)`, C = green `oklch(0.78 0.13 155)`, D = amber `oklch(0.82 0.14 80)`, then repeat. Update Meeting Mode artboard A so half the lines are speaker A, half are speaker B, demonstrating the multi-participant case.

### 2 · Light theme variant
Render the entire app in a light theme alongside the dark. Light palette: invert neutrals (bg ≈ `oklch(0.98 0.003 250)`, surface ≈ `oklch(0.95 0.005 250)`, surface2 ≈ `oklch(0.92 0.006 250)`, etc.), text ≈ `oklch(0.20 0.005 250)`, borders ≈ `oklch(0.85 0.008 250)`. Keep cyan accent at the same chroma but adjust lightness to `oklch(0.55 0.16 205)` so it reads on a light background. REC red `oklch(0.55 0.20 25)`, warn `oklch(0.65 0.16 80)`, ok `oklch(0.55 0.14 155)`. Define the full token table in the v2 bundle so it's reusable.

Deliver light-theme variants of: Meeting active (A), Conversation active (B), Idle (C), Onboarding (D), Overlay active 5-line, Settings sheet. (6 light artboards.)

### 3 · Overlay hover-reveal control strip
v1 had a permanently-visible top bar with state dot, drag handle, pin button, and close button. **Replace with a hover-reveal model**:

- **Default state** (no cursor over the window): only the 5-line text stack is rendered. No chrome at all. The window is fully click-through.
- **Hover state** (cursor enters the window's bounding rect): a thin **24 px** strip slides in from the top edge over a 120 ms fade. Strip contains, left to right: state dot (REC / warn / dim) · state label (`EN → VI` / `RECONNECTING` / `IDLE`, mono 9 px) · flexible drag region (cursor changes to `grab`) · close X button (12 px icon, 22×22 hit area). The strip captures mouse events while visible. After ~1.5 s of no movement over the strip, it fades out again and the window returns to click-through.
- **No pin button** — always-on-top is a permanent property, not a toggle.

Render two artboards for this: (a) hover-state with strip visible, (b) default state with strip hidden.

### 4 · Sidebar date grouping
v1 showed a flat list of sessions. **Add sticky date-group headers** between items so long history is scannable. Headers: `Today`, `Yesterday`, `This week`, `Earlier`. Header style: mono 9 px uppercase letter-spacing 0.6, color `subtle`, 4 px above and below, no separator line. Headers stick to the top of the scroll container as the user scrolls past them. Show 12+ mock sessions across at least 3 groups in the v2 artboard.

### 5 · App icon system
Design 4 variants of the Voxtide app icon, side by side on their own design section:

1. **1024 × 1024 marketing icon** — cyan rounded-square (squircle, 22% corner radius per macOS convention) with the sound-wave glyph centered, white. Add subtle inner highlight and bottom shadow for depth.
2. **32 × 32 list icon** — same composition, simplified glyph (3 bars instead of 5 if the 5-bar version aliases).
3. **16 × 16 list icon** — test legibility. If the sound-wave glyph smears, switch to a single mark: a stylized `V` made of two intersecting wave-curves, or a single downward-curving wave. Pick whichever survives.
4. **macOS menu-bar monochrome (22 × 22, template image)** — single-color silhouette designed to render in any tint (system handles black/white inversion). Just the wave/V shape, no square background.

Deliver each at 1× and 2× side by side so we can judge edge quality.

### 6 · Status bar at narrow width
v1 showed the status bar at full main-window width (920 px). At narrower widths the dense 6-field row will crowd. Render the status bar at three widths:

- **920 px (full)** — all 6 fields.
- **720 px (narrow)** — hide audio format.
- **600 px (very narrow)** — also hide latency.
- **480 px (cramped)** — also hide Soniox model name. Keep REC dot + elapsed + level meter + translation summary.

Stack them vertically as one design section so the priority is visually obvious. Add a post-it noting: "Hide priority, right-to-left: audio format → latency → model → translation summary. Never hide REC + elapsed + meter."

## 4. Out of scope for v2

- Architecture, data model, audio capture, Soniox protocol — settled in `docs/superpowers/specs/2026-05-12-voxtide-design.md`.
- New modes or features beyond what's in v1.
- Onboarding flow changes (the "no API key" state stays as-is in v1).
- Settings sheet additions — only re-render in light theme; structure stays.

## 5. Output format

Same as v1: a `DesignCanvas` with sections, each section containing `DCArtboard`s and `DCPostIt` callouts. Sections to include in v2:

1. **Meeting — speaker chips** (updated artboard A with chips)
2. **Light theme — main window** (A, B, C, D)
3. **Light theme — overlay + settings**
4. **Overlay — hover-reveal control strip** (hover + default)
5. **Sidebar — date grouping**
6. **App icon system** (4 sizes × 1× and 2×)
7. **Status bar — narrow widths** (4 widths stacked)

Geist + Geist Mono, oklch tokens, original aesthetic. Match the visual cohesion of v1.
