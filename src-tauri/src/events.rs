use tauri::{AppHandle, Emitter};
use voxtide_core::session::CoreEvent;

/// Forward a core event to the webview verbatim. `CoreEvent` derives
/// `Serialize` with the wire contract baked in (`tag = "kind"`, kebab-case
/// variant names, snake_case fields), so there is no translation layer: the
/// emitted JSON is exactly what `src/lib/ipc.ts` consumes. This replaced a
/// hand-written `WireEvent` mirror + ~47-line copying match that could silently
/// drift from `CoreEvent`. Shape is pinned by the tests below.
pub fn forward(app: &AppHandle, ev: CoreEvent) {
    let _ = app.emit("voxtide://event", &ev);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use voxtide_core::translation::tokens::TranslationStatus;

    /// Pins the exact on-the-wire JSON for a representative spread of variants:
    /// a struct variant with renamed fields, the `TranscriptFinal` variant whose
    /// `status` (enum) + `chip` (`Option<char>`) types must serialize as the
    /// frontend expects, a unit variant, and the new `Error` variant. If any
    /// serde attribute drifts (e.g. someone adds `rename_all` for fields, or the
    /// tag key changes) these break — protecting `src/lib/ipc.ts`'s union.
    #[test]
    fn wire_shape_is_byte_stable() {
        // Unit variant -> just the kind tag, kebab-cased.
        assert_eq!(
            serde_json::to_value(CoreEvent::UtteranceBreak).unwrap(),
            json!({ "kind": "utterance-break" })
        );

        // Struct variant: snake_case fields preserved, kebab-case kind.
        assert_eq!(
            serde_json::to_value(CoreEvent::SessionStopped {
                session_id: 7,
                duration_ms: 1234,
            })
            .unwrap(),
            json!({ "kind": "session-stopped", "session_id": 7, "duration_ms": 1234 })
        );

        // status (TranslationStatus -> snake_case string) and chip (Option<char>
        // -> 1-char string) are the historically fragile fields.
        assert_eq!(
            serde_json::to_value(CoreEvent::TranscriptFinal {
                status: TranslationStatus::Translation,
                text: "Xin chào".into(),
                language: Some("vi".into()),
                chip: Some('A'),
                ts_ms: 110,
            })
            .unwrap(),
            json!({
                "kind": "transcript-final",
                "status": "translation",
                "text": "Xin chào",
                "language": "vi",
                "chip": "A",
                "ts_ms": 110,
            })
        );

        // None chip serializes as JSON null (Option<char>).
        assert_eq!(
            serde_json::to_value(CoreEvent::TranscriptLive {
                status: TranslationStatus::Original,
                text: "Hel".into(),
                language: None,
                chip: None,
            })
            .unwrap(),
            json!({
                "kind": "transcript-live",
                "status": "original",
                "text": "Hel",
                "language": null,
                "chip": null,
            })
        );

        // ConnectionState uses &'static str + Option fields.
        assert_eq!(
            serde_json::to_value(CoreEvent::ConnectionState {
                state: "reconnecting",
                attempt: Some(2),
                retry_in_ms: Some(500),
            })
            .unwrap(),
            json!({
                "kind": "connection-state",
                "state": "reconnecting",
                "attempt": 2,
                "retry_in_ms": 500,
            })
        );

        // The new error variant.
        assert_eq!(
            serde_json::to_value(CoreEvent::Error {
                message: "Soniox error 401: bad key".into(),
            })
            .unwrap(),
            json!({ "kind": "error", "message": "Soniox error 401: bad key" })
        );
    }
}
