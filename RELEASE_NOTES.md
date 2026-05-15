# Voxtide v0.1.4

### Fixed
- **Translation direction in System Audio mode.** The transcript no longer
  shows the same language in both columns (or an empty Translation pane).
  The two language slots are now simply **Source** (what is spoken) and
  **Target** (the translation); the confusing "YOU / my language" marker has
  been removed. Listening to one language and reading another now just works.

### New
- **Pinyin for Chinese.** Optional pinyin shown above Chinese text. Toggle it
  in Settings → Reading or from the new "Aa" button on each transcript column.
- **Adjustable transcript font size.** Five sizes (XS–XL) from Settings →
  Reading and the "Aa" popover. Your choice is remembered.
- **Pause-based grouping.** A single speaker talking for a long stretch no
  longer turns into one unreadable wall of text — the transcript now breaks
  into separate timestamped chunks at natural speech pauses, in both columns.

### Improved
- Capture modes are now labelled **System Audio** and **Microphone**
  (clearer than "Meeting" / "Conversation").
- The language **swap** control is now an obvious click-to-swap button — a
  circular exchange icon with a hover state and tooltip — and it swaps the
  source and target languages.
- Settings → Default languages now reads **Source language / Target language**.

### Notes
- Existing installs update automatically. Your saved settings carry over
  unchanged — no reconfiguration needed.
- macOS (Apple Silicon) and Windows builds are attached below.
