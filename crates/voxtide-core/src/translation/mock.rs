use tokio::sync::mpsc;

use crate::translation::{SessionConfig, TranslationEvent, TranslationProvider};
use crate::Result;

pub struct MockProvider {
    script: Vec<TranslationEvent>,
    /// Events released ONLY when [`eos`](TranslationProvider::eos) is called —
    /// modelling Soniox's flushed trailing finals that arrive after the client
    /// signals end-of-stream. Empty for the plain `with_script` path, which
    /// keeps the original behaviour (channel closes when the script drains).
    flush_on_eos: Vec<TranslationEvent>,
    rx: Option<mpsc::Receiver<TranslationEvent>>,
    /// Retained sender used to deliver `flush_on_eos` after `eos()`. Held only
    /// when a flush is queued, so it keeps the event channel open past the
    /// script; for the plain path it stays `None` and the channel closes
    /// (yielding `next_event() -> None`) exactly as before.
    flush_tx: Option<mpsc::Sender<TranslationEvent>>,
}

impl MockProvider {
    pub fn with_script(script: Vec<TranslationEvent>) -> Self {
        Self::with_script_and_flush(script, Vec::new())
    }

    /// Like [`with_script`](Self::with_script) but additionally queues
    /// `flush_on_eos` events that are withheld until `eos()` is called. Use this
    /// to exercise the worker's EOS-drain path (trailing finals on explicit
    /// stop).
    ///
    /// **Interleaving caveat:** flush/script interleaving is unspecified if the
    /// script is still being produced (i.e. the spawned sender task has not yet
    /// finished) when `eos()` fires. The flush events are injected immediately on
    /// `eos()` via a retained sender clone, so they can arrive interleaved with
    /// in-flight script events. Tests that care about ordering should ensure the
    /// script has drained before calling `eos()` (or rely on the session worker's
    /// natural sequencing of `eos()` after the stop arm fires).
    ///
    /// **Lifetime caveat:** a provider constructed with a non-empty flush queue
    /// whose session never calls `eos()` will park `next_event()` forever once
    /// the script drains — the retained `flush_tx` keeps the channel open
    /// indefinitely. This is intentional for tests that *require* `eos()` to be
    /// called; do not use `with_script_and_flush` in tests where `eos()` is
    /// never expected to fire.
    pub fn with_script_and_flush(
        script: Vec<TranslationEvent>,
        flush_on_eos: Vec<TranslationEvent>,
    ) -> Self {
        Self {
            script,
            flush_on_eos,
            rx: None,
            flush_tx: None,
        }
    }
}

#[async_trait::async_trait]
impl TranslationProvider for MockProvider {
    async fn open(&mut self, _cfg: SessionConfig) -> Result<()> {
        let (tx, rx) = mpsc::channel(64);
        let script = std::mem::take(&mut self.script);
        // Only retain a sender clone when there are flush events to release on
        // eos(); otherwise the script task's sender is the sole producer and the
        // channel closes when it drains — preserving the legacy with_script
        // semantics (next_event() -> None terminates the worker loop).
        if !self.flush_on_eos.is_empty() {
            self.flush_tx = Some(tx.clone());
        }
        tokio::spawn(async move {
            for ev in script {
                if tx.send(ev).await.is_err() {
                    return;
                }
            }
        });
        self.rx = Some(rx);
        Ok(())
    }
    async fn send_audio(&mut self, _pcm: Vec<u8>) -> Result<()> {
        Ok(())
    }
    async fn next_event(&mut self) -> Option<TranslationEvent> {
        self.rx.as_mut()?.recv().await
    }
    async fn eos(&mut self) {
        // Release the queued trailing finals, then drop the retained sender so
        // the channel closes once they're consumed. Idempotent: `flush_tx.take()`
        // yields `None` on a second call, sending nothing.
        if let Some(tx) = self.flush_tx.take() {
            let flush = std::mem::take(&mut self.flush_on_eos);
            for ev in flush {
                if tx.send(ev).await.is_err() {
                    return;
                }
            }
        }
    }
    async fn close(&mut self) -> Result<()> {
        self.rx = None;
        self.flush_tx = None;
        Ok(())
    }
}
