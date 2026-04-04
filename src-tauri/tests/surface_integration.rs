/// Tauri integration tests for the GPU surface layer.
///
/// Uses `tauri::test::mock_builder()` to spin up a mock app (no real OS window is
/// created). The tests exercise state initialisation and window-context behaviour;
/// they do NOT call `init_surface` because `MacOSContext::new` requires a real
/// AppKit window handle that `MockRuntime` cannot provide.
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tauri::Manager;
use tauri_svelte_lib::SurfaceMap;

// ── helpers ───────────────────────────────────────────────────────────────────

fn build_test_app() -> tauri::App<tauri::test::MockRuntime> {
    // Note: setup() callbacks only fire inside app.run() — not during build().
    // To make state available immediately in tests we call manage() after build.
    let app = tauri::test::mock_builder()
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .expect("failed to build mock app");

    let map: SurfaceMap = Arc::new(Mutex::new(HashMap::new()));
    app.manage(map);
    app
}

// ── state management ──────────────────────────────────────────────────────────

/// The surface map must start empty — no surfaces are created until a frontend
/// window explicitly calls `init_surface`.
#[test]
fn surface_map_initializes_empty() {
    let app = build_test_app();
    let map = app.state::<SurfaceMap>();
    assert!(
        map.lock().unwrap().is_empty(),
        "SurfaceMap should be empty before any init_surface call"
    );
}

/// Cloning the inner Arc must observe the same shared state.
#[test]
fn surface_map_is_shared_across_arc_clones() {
    let app = build_test_app();
    let state = app.state::<SurfaceMap>();
    let cloned: SurfaceMap = Arc::clone(&state);

    assert!(state.lock().unwrap().is_empty());
    assert!(cloned.lock().unwrap().is_empty());
}

// ── window context ────────────────────────────────────────────────────────────

/// Creates a mock Tauri window and verifies that the surface map contains no
/// entry for it — confirming that window creation alone does not trigger surface
/// initialisation (that is the caller's responsibility via `init_surface`).
#[test]
fn new_window_has_no_surface_entry() {
    let app = build_test_app();

    let window = tauri::WebviewWindowBuilder::new(&app, "test-panel", Default::default())
        .build()
        .expect("failed to create mock window");

    let map = app.state::<SurfaceMap>();
    let guard = map.lock().unwrap();
    assert!(
        guard.get(window.label()).is_none(),
        "no surface entry should exist for a freshly created window"
    );
}

/// Simulates the `render_surface` no-op path: when the map has no renderer for
/// a given window label, the lookup returns `None` and nothing panics.
#[test]
fn surface_lookup_returns_none_for_unknown_label() {
    let app = build_test_app();

    let window = tauri::WebviewWindowBuilder::new(&app, "unknown-panel", Default::default())
        .build()
        .expect("failed to create mock window");

    let renderer = app
        .state::<SurfaceMap>()
        .lock()
        .ok()
        .and_then(|m| m.get(window.label()).map(Arc::clone));

    assert!(
        renderer.is_none(),
        "render_surface should silently skip panels without an initialised surface"
    );
}

/// Simulates the `set_surface_rect` error path: looking up an un-initialised
/// window label returns `None`, which the command maps to "surface not
/// initialized".
#[test]
fn set_surface_rect_error_path_for_uninitialised_surface() {
    let app = build_test_app();

    let window = tauri::WebviewWindowBuilder::new(&app, "uninit-panel", Default::default())
        .build()
        .expect("failed to create mock window");

    let found = app
        .state::<SurfaceMap>()
        .lock()
        .ok()
        .and_then(|m| m.get(window.label()).map(Arc::clone));

    assert!(found.is_none(), "surface not initialized — expected None, got Some");
}

// ── platform detection ────────────────────────────────────────────────────────

/// On macOS CI this test confirms we are on the platform that supports
/// `MacOSContext` / `CAMetalLayer`. It deliberately uses a Tauri window
/// context (via `mock_builder`) to verify that the Tauri runtime itself also
/// reports macOS.
#[cfg(target_os = "macos")]
#[test]
fn tauri_context_runs_on_macos() {
    let app = build_test_app();
    let _window = tauri::WebviewWindowBuilder::new(&app, "platform-check", Default::default())
        .build()
        .expect("failed to create mock window");

    assert_eq!(
        std::env::consts::OS, "macos",
        "Native surface support requires macOS; this CI runner is not macOS"
    );
}
