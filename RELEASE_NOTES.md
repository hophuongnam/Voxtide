# Voxtide v0.1.5

### Fixed
- **Stuck "recording" history entries can now be deleted.** Some past
  sessions showed a red recording dot with no delete button and a "—"
  duration, and were impossible to remove. These were sessions that never
  got finalized — for example when the app was quit or the connection
  dropped while recording. They are now normal history entries: no false
  recording dot, and deletable like any other.
- **Sessions are always finalized.** A session now records its end time on
  every path — when you stop it, when you quit the app mid-recording, and
  when the transcription connection closes unexpectedly — so no new stuck
  entries can accumulate.
- **Existing stuck entries are repaired automatically** the next time you
  launch this version. Nothing to do on your end.

### Notes
- Existing installs update automatically. Your saved settings and history
  carry over unchanged.
- macOS (Apple Silicon) and Windows builds are attached below.
