use tokio::sync::mpsc;

use crate::translation::{SessionConfig, TranslationEvent, TranslationProvider};
use crate::Result;

pub struct MockProvider {
    script: Vec<TranslationEvent>,
    rx: Option<mpsc::Receiver<TranslationEvent>>,
}

impl MockProvider {
    pub fn with_script(script: Vec<TranslationEvent>) -> Self {
        Self { script, rx: None }
    }
}

#[async_trait::async_trait]
impl TranslationProvider for MockProvider {
    async fn open(&mut self, _cfg: SessionConfig) -> Result<()> {
        let (tx, rx) = mpsc::channel(64);
        let script = std::mem::take(&mut self.script);
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
    async fn close(&mut self) -> Result<()> {
        self.rx = None;
        Ok(())
    }
}
