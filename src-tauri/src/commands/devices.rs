use serde::Serialize;

#[derive(Serialize)]
pub struct DeviceEntry {
    pub id: String,
    pub label: String,
    pub default: bool,
}

#[tauri::command]
pub fn list_mics() -> Result<Vec<DeviceEntry>, String> {
    voxtide_core::audio::mic::list_input_devices()
        .map(|v| {
            v.into_iter()
                .map(|d| DeviceEntry { id: d.id, label: d.label, default: d.default })
                .collect()
        })
        .map_err(|e| e.to_string())
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn list_loopback_sources() -> Result<Vec<DeviceEntry>, String> {
    voxtide_core::audio::macos_loopback::list_loopback_sources()
        .map(|v| {
            v.into_iter()
                .map(|s| DeviceEntry { id: s.id, label: s.label, default: false })
                .collect()
        })
        .map_err(|e| e.to_string())
}

#[cfg(target_os = "windows")]
#[tauri::command]
pub fn list_loopback_sources() -> Result<Vec<DeviceEntry>, String> {
    voxtide_core::audio::windows_loopback::list_loopback_sources()
        .map(|v| {
            v.into_iter()
                .map(|s| DeviceEntry { id: s.id, label: s.label, default: s.default })
                .collect()
        })
        .map_err(|e| e.to_string())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
#[tauri::command]
pub fn list_loopback_sources() -> Result<Vec<DeviceEntry>, String> {
    Ok(Vec::new())
}
