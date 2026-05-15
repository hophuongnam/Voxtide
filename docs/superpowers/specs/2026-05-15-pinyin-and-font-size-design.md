# Pinyin Furigana + Transcript Font Size — Design

**Date:** 2026-05-15
**Status:** Approved — ready for implementation plan
**Scope:** Live transcript pane + past-session viewer. Overlay window excluded.

## Problem

When a transcript line contains Chinese (`line.language === 'zh'`), readers who
are still learning Mandarin can't easily decode unfamiliar characters. Voxtide
already tags each line with a language code, but renders the raw text directly
in `Line.svelte`. There is no per-character pronunciation aid.

Separately, the transcript text is fixed at `text-[13.5px]` and cannot be
resized without editing the source.

## Goals

1. Render pinyin above Han characters as ruby annotations when the user opts in.
2. Provide a font-size control with five discrete steps (XS / S / M / L / XL).
3. Both settings persist across app restarts via the existing `AppConfig` flow.
4. Apply to live transcript and past-session viewer. Skip the floating overlay.
5. Off by default — pinyin disabled, font-size at M (current 13.5 px).

## Non-goals

- Japanese kanji → hiragana furigana. Only `language === 'zh'` is gated.
- Per-pane independent font sizes. One shared slider drives both columns.
- Overlay-window styling. Out of scope by design.
- Word-segmentation rendering (e.g. `<ruby>北京<rt>běi jīng</rt></ruby>`).
  Per-character rendering is simpler and survives mid-phrase live updates.

## Approach

**Frontend-only**, using `pinyin-pro` (~50 KB gzipped, dictionary-based
polyphone resolution, actively maintained). A new `<RubyText text={…} />`
component slices the input into runs of Han vs non-Han codepoints and emits
`<ruby>字<rt>zì</rt></ruby>` for Han runs only. Non-Han runs (ASCII,
punctuation, kana, hangul) pass through as plain text.

`Line.svelte` branches:

```svelte
{#if showPinyin && line.language === 'zh'}
  <RubyText text={line.text} />
{:else}
  {line.text}
{/if}
```

Font size lives in a CSS custom property `--vt-transcript-size` set on the
`TranscriptPane` root. `Line.svelte` reads `font-size: var(--vt-transcript-size)`
instead of the hardcoded `text-[13.5px]`. Ruby annotation size uses `0.5em` so
it scales with the inherited base size — no separate ruby slider.

Alternatives considered:

- **Post-render DOM walk:** Wrap Han chars after Svelte renders. Rejected —
  fights Svelte's diffing on every live partial update and breaks unit testing.
- **Rust-side annotation:** Annotate in `voxtide-core` and ship `[{char, pinyin}]`
  arrays over IPC. Rejected — changes the wire format for every transcript event;
  invasive; recomputes for stale lines a user never opens.

## Persistence schema

Two new fields on `AppConfig`, both Rust and TS sides. Both use
`#[serde(default)]` so existing `config.json` files load unchanged.

```rust
// crates/voxtide-core/src/config.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum FontSize {
    Xs,
    S,
    #[default]
    M,
    L,
    Xl,
}

pub struct AppConfig {
    // ... existing fields ...

    #[serde(default)]
    pub font_size: FontSize,

    #[serde(default)]
    pub show_pinyin: bool,
}
```

TS mirror in `src/types.ts`:

```ts
export type FontSize = 'xs' | 's' | 'm' | 'l' | 'xl';

export interface AppConfig {
  // ... existing fields ...
  font_size: FontSize;
  show_pinyin: boolean;
}
```

Both fields persist through the existing `setConfig(next)` + `config.setConfig(next)`
pattern used by every other per-control change in `MainApp.svelte`.

Pixel mapping lives in TS only:

```ts
const fontSizePx: Record<FontSize, string> = {
  xs: '11px',
  s:  '12.5px',
  m:  '13.5px',
  l:  '16px',
  xl: '19px',
};
```

## Components

### New files

- **`src/lib/pinyin.ts`** — wraps `pinyin-pro`. Public API:
  ```ts
  export interface PinyinChar { char: string; pinyin: string }
  export function toPinyin(text: string): PinyinChar[];
  ```
  Returns one `PinyinChar` per Unicode codepoint of the input. For Han
  codepoints, `pinyin` is the tone-marked syllable. For non-Han codepoints,
  `pinyin` is the empty string `''`. The full input is passed to `pinyin-pro`
  in one call so its polyphone context engine sees surrounding characters
  (including non-Han ones) — slicing the string before the lookup would defeat
  that. Backed by an LRU cache (max 256 entries, keyed by the full input string).

- **`src/components/transcript/RubyText.svelte`** — props: `text: string`.
  Calls `toPinyin(text)` once, then iterates the result: when `pinyin === ''`
  emits a plain text node; otherwise emits `<ruby>{char}<rt>{pinyin}</rt></ruby>`.
  Consecutive non-Han chars coalesce into a single text node for cleaner DOM.
  Falls back to plain `{text}` on any thrown error from `toPinyin`.

- **`src/components/transcript/ReadingControls.svelte`** — small popover panel:
  five font-size buttons (XS / S / M / L / XL, current size highlighted), a "拼
  Show pinyin" toggle row. Calls `onchange(next: AppConfig)`.

- **`src/components/settings/ReadingSection.svelte`** — same content as
  `ReadingControls` but in the Settings-sheet section style (matching
  `AppearanceSection`).

### Modified files

- **`src/components/transcript/Line.svelte`** — accepts `showPinyin: boolean` as
  a prop. Branches between `RubyText` and plain text. Replaces `text-[13.5px]`
  with `style:font-size="var(--vt-transcript-size)"`.

- **`src/components/transcript/Column.svelte`** — adds an "Aa" button
  (right-aligned in the 38 px header) that toggles a `ReadingControls` popover
  anchored below it. Both columns show the button; both write the same shared
  config.

- **`src/components/transcript/TranscriptPane.svelte`** — accepts
  `fontSize: FontSize`, `showPinyin: boolean`, `onconfigchange: (next: AppConfig) => void`
  as new props. Sets `style:--vt-transcript-size={fontSizePx[fontSize]}` on its
  root div. Threads `showPinyin` to each `<Line>` and `onconfigchange` to each
  `<Column>`.

- **`src/components/settings/SettingsSheet.svelte`** — slots `<ReadingSection>`
  between `AppearanceSection` and the other sections.

- **`src/routes/MainApp.svelte`** — reads `cfg.font_size` and `cfg.show_pinyin`
  from `config.config`, passes both `<TranscriptPane>` instances (live + past
  viewer). New handler `onReadingChange(next)` follows the existing
  `await setConfig(next); config.setConfig(next)` pattern.

- **`src/types.ts`** — adds `FontSize` type and the two new `AppConfig` fields.

- **`crates/voxtide-core/src/config.rs`** — adds `FontSize` enum and the two new
  fields with `#[serde(default)]`.

## Data flow

1. **Boot:** `MainApp.onMount` → `getConfig()` → `config.setConfig(cfg)`.
   New fields ride along with the rest of `AppConfig`.
2. **Read:** Both `<TranscriptPane>` mounts read `cfg.font_size` and
   `cfg.show_pinyin` from the config store and pass as props. `TranscriptPane`
   sets the CSS variable on its root. `Line.svelte` reads it via `var()`.
3. **Render:** `Line.svelte` branches on `showPinyin && line.language === 'zh'`.
   Live partial lines (which have `language` set in `TranscriptPane`'s inline
   construction) flow through the same branch.
4. **Write:** `ReadingControls` (in the popover) or `ReadingSection` (in
   Settings) calls `onchange(next)`. The handler bubbles up to `MainApp`'s
   `onReadingChange`, which calls `await setConfig(next)` then
   `config.setConfig(next)` — same two-line pattern as `onLangPick`,
   `onModeChange`, etc.

## `pinyin-pro` bundling

Eager static import in `lib/pinyin.ts`. Lazy-loading would cause a one-frame
flicker on first toggle-on, and 50 KB is negligible for a desktop bundle. The
LRU cache sits in front of the library call so repeated phrases during a long
session amortize lookup cost.

## Error handling & edge cases

**Config decode:**
- Old `config.json` lacks the two new fields. `#[serde(default)]` returns `M`
  and `false`. No migration code.
- Invalid enum string from a hand-edited file: serde rejects, `ConfigStore::load`
  falls back to `AppConfig::default()` (existing behavior).

**Pinyin lookups:**
- Empty / whitespace → `toPinyin` returns `[]`. `RubyText` emits nothing.
- Non-Han codepoint → `pinyin === ''` in the returned `PinyinChar`. `RubyText`
  emits a plain text node, no `<rt>`.
- Unrecognized Han codepoint (rare CJK Extension B+) → `pinyin-pro` returns the
  char itself. The wrapper in `lib/pinyin.ts` post-processes: when the returned
  pinyin equals the input char, it sets `pinyin = ''` so `RubyText` renders
  that char plain (no empty `<rt>`).
- Library throws → wrap the whole `toPinyin` call in try/catch; on error,
  return `[{char: text, pinyin: ''}]` (one plain-text entry). `RubyText`'s own
  try/catch is a belt-and-suspenders second layer. Transcript must never
  become unreadable due to pinyin failure.

**Language gating:**
- Only `line.language === 'zh'` triggers ruby rendering. Mixed-language lines
  (zh with code-switched English) work because `RubyText` skips non-Han runs
  naturally.
- `language: null` (very old past sessions before language tagging) → toggle
  has no effect; renders plain. Acceptable, intentional.

**Live partial lines:**
- Soniox emits partial text on every audio chunk. The LRU cache in `toPinyin`
  ensures repeated prefixes don't re-tokenize. 256-entry cap keeps memory
  bounded (~tens of KB).

**Font-size CSS variable:**
- At XS (11 px), `<rt>` becomes 5.5 px — at the edge of readability, but the
  user opted in. No clamping.

**Popover dismiss:**
- Outside-click closes (matches `SettingsSheet`). Escape closes. Focus returns
  to the "Aa" button.

**Accessibility:**
- `<ruby>` / `<rt>` is well-supported by screen readers (read the base text,
  skip rt). No ARIA changes needed.

## Testing

### Unit tests (Vitest)

- **`src/lib/pinyin.test.ts`:**
  - `toPinyin('你好')` → `[{char:'你',pinyin:'nǐ'},{char:'好',pinyin:'hǎo'}]`
  - `toPinyin('Hi世界')` → `[{char:'H',pinyin:''},{char:'i',pinyin:''},{char:'世',pinyin:'shì'},{char:'界',pinyin:'jiè'}]`
  - Polyphone: `toPinyin('银行')[1].pinyin === 'háng'` (not `'xíng'` — verifies
    context engine)
  - `toPinyin('')` → `[]`
  - LRU: same input twice → underlying `pinyin-pro` call invoked once (assert
    via spy)

- **`src/components/transcript/RubyText.test.ts`** (`@testing-library/svelte`):
  - Han-only input emits the expected `<ruby><rt>` structure
  - Mixed input renders ASCII as text nodes, Han as ruby
  - Empty input renders nothing
  - Library throw → output is plain text, no `<ruby>` in DOM

### Rust tests (`crates/voxtide-core/tests/config.rs`)

- Load JSON missing `font_size` + `show_pinyin` → defaults applied.
- Round-trip with both fields set → values preserved.

### Component integration

- `ReadingSection`: font-size button click → `onchange` fires with `next.font_size`
  set correctly.
- `Column`: "Aa" click opens popover; toggle change → `onconfigchange` fires
  with `next.show_pinyin` flipped.
- Tauri plugin guard per `feedback_tauri_plugin_jsdom_guard`: mock
  `__TAURI_INTERNALS__` in jsdom env.

### Manual smoke gate

Mandatory before claiming "complete" per the Voxtide rule
(`feedback_voxtide_smoke_before_complete`).

Existing 10 checks must still pass (regression): boot, start click, tokens
land, partial updates, mode switch, dual-scroll sync, follow-tail, past-viewer,
markers, overlay window.

Five new checks for this feature:

11. Toolbar "Aa" → popover opens → step the font size; both panes resize live;
    restart app → size persists.
12. Start a zh capture → toggle pinyin ON → live partial gets ruby annotations
    without flicker.
13. Open a past zh session → pinyin ON shows ruby, OFF shows plain.
14. Line containing `Hello世界` → English renders plain, Chinese annotated.
15. Manually edit `config.json` to remove the two new fields → app loads with
    defaults (M / off); toolbar reads the correct state.

## Rollout

This is a single PR with no flag-gate. The defaults (`M`, `false`) make the
feature invisible to users who don't opt in, so safe to ship straight to main
after the smoke gate passes.

## Open questions

None — locked during brainstorming session.

## Related memories

- `feedback_voxtide_persist_every_ui_pick` — every UI selection must persist
  immediately via `setConfig`. Followed by the `onReadingChange` flow.
- `feedback_voxtide_smoke_before_complete` — 10-check smoke gate mandatory.
  Extended with five new checks here.
- `feedback_tauri_plugin_jsdom_guard` — guard plugin invokes in jsdom tests.
- `reference_voxtide_build_artifact_paths` — bundle location reminder, not
  load-bearing for this feature.
