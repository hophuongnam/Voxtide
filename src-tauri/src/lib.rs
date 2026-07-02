use tauri::Manager;

mod commands;
mod events;
#[cfg(target_os = "macos")]
mod overlay_follow;
#[cfg(desktop)]
mod hotkey;
mod state;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Deliberately NOT #[tokio::main]: the ExitRequested handler below calls
    // tauri::async_runtime::block_on, which panics when invoked from inside a
    // tokio runtime context ("Cannot start a runtime from within a runtime").
    // Building the runtime manually and handing Tauri its handle keeps main a
    // plain thread, so the quit-path block_on (session finalize) is legal.
    let rt = tokio::runtime::Runtime::new().expect("failed to build tokio runtime");
    tauri::async_runtime::set(rt.handle().clone());

    tracing_subscriber::fmt::try_init().ok();

    let mut builder = tauri::Builder::default();
    #[cfg(desktop)]
    {
        // MUST be the first plugin: a second launch is killed during plugin
        // initialization, before .setup() opens the shared DB below. Without
        // the guard, the second instance's Store::open reconcile stamped the
        // FIRST instance's live session as ended (ended_at = started_at,
        // duration 0); on Windows it also panicked on the hotkey conflict.
        // The callback runs in the FIRST instance: surface its window.
        builder = builder
            .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }))
            .plugin(tauri_plugin_global_shortcut::Builder::new().build())
            .plugin(tauri_plugin_updater::Builder::new().build())
            .plugin(tauri_plugin_process::init())
            // Auto-persist main window size + position to ~/Library/Application Support/Voxtide/.window-state.
            // The overlay window is denylisted: it has a fixed-by-design hover-strip layout that
            // shouldn't be perturbed by restored state from a prior session.
            .plugin(
                tauri_plugin_window_state::Builder::default()
                    .with_denylist(&["overlay"])
                    .build(),
            );
    }
    builder = builder.plugin(tauri_plugin_opener::init());
    let app = builder
        .on_window_event(|_window, _event| {
            // macOS pattern: red-traffic-light closes the window but keeps the app
            // running in the dock; dock-click re-shows it (handled in the run loop).
            // Cmd+Q / "Quit Voxtide" fire ExitRequested, not CloseRequested, so quit
            // still works as expected.
            //
            // macOS-ONLY: Windows/Linux have no Dock or tray affordance here, so
            // close-to-hide would strand an invisible process whose global hotkey
            // can still start recordings (only Task Manager could kill it). On
            // those platforms the close proceeds → ExitRequested → finalize.
            #[cfg(target_os = "macos")]
            if let tauri::WindowEvent::CloseRequested { api, .. } = _event {
                if _window.label() == "main" {
                    api.prevent_close();
                    let _ = _window.hide();
                }
            }
        })
        .setup(move |app| {
            // DB open + orphan reconcile live INSIDE setup so they can never
            // run in a doomed second instance (the single-instance plugin
            // exits one during plugin init, which precedes setup). block_on
            // is legal here: setup runs on the plain (non-async) main thread
            // — see the runtime comment at the top of main().
            let dir = state::data_dir(app.handle());
            let state = tauri::async_runtime::block_on(state::init(dir))?;
            // Subscribe BEFORE handing state to Tauri so we hold a reference to the controller.
            // This single persistent forwarder replaces the per-call spawns that were previously
            // in `lifecycle::start_session`, which leaked one task per start/stop cycle.
            let mut rx = state.controller.subscribe();
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    match rx.recv().await {
                        Ok(ev) => crate::events::forward(&app_handle, ev),
                        // Lagged: some events were dropped because we fell behind the sender.
                        // Treat as a refresh signal — continue rather than breaking.
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                        // Channel closed (controller dropped). Forwarder task can exit.
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    }
                }
            });
            app.manage(state);
            // Register the configured global hotkey. Failure is NON-FATAL —
            // the app is fully usable without it, and an OS-level conflict
            // with another app's shortcut is an expected runtime condition.
            // (A `?` here aborted the whole startup.)
            #[cfg(desktop)]
            {
                let accel = app
                    .state::<state::AppState>()
                    .config
                    .load()
                    .map(|c| c.hotkey)
                    .unwrap_or_else(|_| voxtide_core::config::AppConfig::default().hotkey);
                if let Err(e) = hotkey::register(app.handle(), &accel) {
                    tracing::warn!(?e, accel = %accel, "global hotkey registration failed; continuing without a hotkey");
                }
            }
            // Overlay is a desktop-only secondary window, created at runtime here
            // instead of statically in tauri.conf.json: a static second window
            // hijacks the single WebView surface on Android (the overlay rendered
            // in place of `main`). Properties mirror the former conf entry.
            #[cfg(desktop)]
            {
                let overlay = tauri::WebviewWindowBuilder::new(
                    app,
                    "overlay",
                    tauri::WebviewUrl::App("overlay.html".into()),
                )
                .title("Voxtide overlay")
                .inner_size(600.0, 190.0)
                .decorations(false)
                .resizable(false)
                .always_on_top(true)
                .skip_taskbar(true)
                .transparent(true)
                .shadow(false)
                .visible(false)
                // Follow the user across Spaces (macOS: CanJoinAllSpaces;
                // no-op on Windows). always_on_top alone doesn't cross Spaces.
                .visible_on_all_workspaces(true)
                .build()?;
                // CanJoinAllSpaces still excludes OTHER apps' fullscreen
                // Spaces; captions over a fullscreen video/presentation also
                // need FullScreenAuxiliary, which tauri/tao never set. OR it
                // in (read-modify-write preserves CanJoinAllSpaces).
                #[cfg(target_os = "macos")]
                {
                    if let Ok(ptr) = overlay.ns_window() {
                        use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior};
                        // SAFETY: ptr is the live NSWindow of the window built
                        // above; setup runs on the main thread, as AppKit requires.
                        unsafe {
                            let ns = &*ptr.cast::<NSWindow>();
                            ns.setCollectionBehavior(
                                ns.collectionBehavior()
                                    | NSWindowCollectionBehavior::FullScreenAuxiliary,
                            );
                        }
                    }
                    // Those flags only cover Spaces on the overlay's own
                    // display; follow the cursor's screen so captions reach
                    // fullscreen apps on OTHER monitors too.
                    overlay_follow::register(&overlay);
                }
                #[cfg(not(target_os = "macos"))]
                let _ = overlay;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::keychain::has_api_key,
            commands::keychain::set_api_key,
            commands::keychain::clear_api_key,
            commands::config::get_config,
            commands::config::set_config,
            commands::config::app_info,
            commands::devices::list_mics,
            commands::devices::list_loopback_sources,
            commands::devices::feed_mic_pcm,
            commands::sessions::list_sessions,
            commands::sessions::get_session,
            commands::sessions::search_transcripts,
            commands::sessions::delete_session,
            commands::lifecycle::start_session,
            commands::lifecycle::stop_session,
            commands::lifecycle::update_context,
            #[cfg(desktop)] commands::overlay::show_overlay,
            #[cfg(desktop)] commands::overlay::hide_overlay,
            #[cfg(desktop)] commands::overlay::set_overlay_click_through,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| {
        match event {
            // Cmd+Q / "Quit Voxtide" / dock-quit fire ExitRequested. Stop the
            // active session first so its row is finalized (ended_at written);
            // otherwise quitting mid-recording orphans it as a permanent
            // "recording" ghost with no delete affordance. stop() waits on the
            // worker join (<=5s); the store's open-time reconcile is the
            // backstop for a hard kill that never reaches this handler.
            tauri::RunEvent::ExitRequested { .. } => {
                if let Some(state) = app_handle.try_state::<state::AppState>() {
                    let controller = state.controller.clone();
                    tauri::async_runtime::block_on(async move {
                        let _ = controller.stop().await;
                    });
                }
            }
            #[cfg(target_os = "macos")]
            tauri::RunEvent::Reopen { .. } => {
                // Deliberately ignore has_visible_windows: a visible OVERLAY
                // counts as a visible window, which blocked Dock-click from
                // ever restoring a hidden main window. Judge the main window's
                // own visibility instead.
                if let Some(window) = app_handle.get_webview_window("main") {
                    if !window.is_visible().unwrap_or(false) {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
            _ => {}
        }
    });
}
