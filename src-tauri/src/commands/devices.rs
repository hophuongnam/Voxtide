use serde::Serialize;

#[derive(Serialize)]
pub struct DeviceEntry {
    pub id: String,
    pub label: String,
    pub default: bool,
}

#[tauri::command]
pub fn list_mics() -> Result<Vec<DeviceEntry>, String> {
    // ponytail: Android audio is deferred to the Phase 0 spike. cpal's Android
    // backend calls `ndk_context::android_context()`, which is uninitialized in a
    // Tauri-mobile app (no ndk-glue) and panic-ABORTS the process (the panic can't
    // unwind across the JNI/IPC boundary). Empty list lets the main UI render;
    // real capture (cpal Path A / getUserMedia Path B) lands in Phase 0.6.
    #[cfg(mobile)]
    {
        Ok(Vec::new())
    }
    #[cfg(desktop)]
    {
        voxtide_core::audio::mic::list_input_devices()
            .map(|v| {
                v.into_iter()
                    .map(|d| DeviceEntry {
                        id: d.id,
                        label: d.label,
                        default: d.default,
                    })
                    .collect()
            })
            .map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub fn list_loopback_sources() -> Result<Vec<DeviceEntry>, String> {
    // System Audio / loopback is desktop-only by product scope, and the cpal path
    // would hit the same Android `ndk_context` panic — return empty on mobile.
    #[cfg(mobile)]
    {
        Ok(Vec::new())
    }
    #[cfg(desktop)]
    {
        voxtide_core::audio::loopback::list()
            .map(|v| {
                v.into_iter()
                    .map(|s| DeviceEntry {
                        id: s.id,
                        label: s.label,
                        default: s.default,
                    })
                    .collect()
            })
            .map_err(|e| e.to_string())
    }
}
