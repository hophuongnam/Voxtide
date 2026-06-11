use serde::Serialize;
use tauri::State;

use voxtide_core::persistence::sessions::{SessionRow, Sessions};
use voxtide_core::persistence::tokens::{TokenRow, Tokens};

use crate::state::AppState;

#[derive(Serialize)]
pub struct SessionWithTokens {
    pub session: SessionRow,
    pub tokens: Vec<TokenRow>,
}

#[tauri::command]
pub async fn list_sessions(
    state: State<'_, AppState>,
    limit: i64,
) -> Result<Vec<SessionRow>, String> {
    let pool = state.controller.store().pool().clone();
    Sessions::list(&pool, limit)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_session(state: State<'_, AppState>, id: i64) -> Result<SessionWithTokens, String> {
    let pool = state.controller.store().pool().clone();
    let session = Sessions::get(&pool, id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("session {id} not found"))?;
    let tokens = Tokens::list_by_session(&pool, id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(SessionWithTokens { session, tokens })
}

#[tauri::command]
pub async fn search_transcripts(
    state: State<'_, AppState>,
    query: String,
    limit: i64,
) -> Result<Vec<SessionRow>, String> {
    let pool = state.controller.store().pool().clone();
    Tokens::search_sessions(&pool, &query, limit)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_session(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    if state.controller.active_session_id() == Some(id) {
        return Err("cannot delete an active session".into());
    }
    let pool = state.controller.store().pool().clone();
    // We intentionally discard the bool — both "row removed" and "row not found"
    // mean the same thing from the caller's perspective: the id is gone. The
    // frontend just refreshes the list either way.
    Sessions::delete(&pool, id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
