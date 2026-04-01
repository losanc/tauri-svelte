use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use native_tauri_surface::{SurfaceResizer, surface_helper::native::SurfaceHelper};
use wgpu_renderer::Renderer;
use tauri::Manager;

type SurfaceMap = Arc<Mutex<HashMap<String, (Renderer, SurfaceResizer)>>>;

/// Create a native wgpu surface for the calling window.
/// Idempotent — safe to call even if the surface already exists.
#[tauri::command]
fn init_surface(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    surfaces: tauri::State<'_, SurfaceMap>,
) -> Result<(), String> {
    let label = window.label().to_string();
    if surfaces.lock().unwrap().contains_key(&label) {
        return Ok(());
    }
    let map = Arc::clone(&surfaces);
    app.run_on_main_thread(move || {
        let (tauri_surface, resizer) = SurfaceHelper::new(&window, 1, 1, 0, 0);
        let renderer = pollster::block_on(Renderer::new(tauri_surface));
        map.lock().unwrap().insert(label, (renderer, resizer));
    })
    .map_err(|e| format!("{e:?}"))
}

#[tauri::command]
fn set_surface_rect(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    surfaces: tauri::State<'_, SurfaceMap>,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), String> {
    let (resizer, frame) = {
        let map = surfaces.lock().unwrap();
        let (renderer, resizer) =
            map.get(window.label()).ok_or("surface not initialized")?;

        // ✅ clone the NSView safely
        let resizer = SurfaceResizer {
            ns_view: resizer.ns_view.clone(),
        };

        if width <= 0.0 || height <= 0.0 {
            return app
                .run_on_main_thread(move || unsafe {
                    resizer.hide();
                })
                .map_err(|e| format!("{e:?}"));
        }

        let scale = window.scale_factor().map_err(|e| e.to_string())?;
        let inner_size = window.inner_size().map_err(|e| e.to_string())?;
        let window_height = inner_size.height as f64 / scale;

        renderer.resize((width * scale) as u32, (height * scale) as u32);

        (resizer, (x, y, width, height, window_height))
    };

    app.run_on_main_thread(move || unsafe {
        let (x, y, w, h, wh) = frame;
        resizer.update_frame(x, y, w, h, wh);
    })
    .map_err(|e| format!("{e:?}"))
}

#[tauri::command]
fn render_surface(window: tauri::WebviewWindow, surfaces: tauri::State<'_, SurfaceMap>) {
    if let Ok(map) = surfaces.lock() {
        if let Some((renderer, _)) = map.get(window.label()) {
            renderer.render();
        }
    }
}

/// Push a resize cursor onto the macOS cursor stack.
/// Pushed cursors take precedence over NSWindow cursor rects (including
/// WKWebView's), so this reliably shows the resize icon regardless of what
/// WKWebView thinks the CSS cursor should be.
#[tauri::command]
fn push_resize_cursor(app: tauri::AppHandle, horizontal: bool) -> Result<(), String> {
    app.run_on_main_thread(move || {
        #[cfg(target_os = "macos")]
        native_tauri_surface::surface_helper::native::push_resize_cursor(horizontal);
    })
    .map_err(|e| format!("{e:?}"))
}

/// Pop the top cursor from the macOS cursor stack.
#[tauri::command]
fn pop_resize_cursor(app: tauri::AppHandle) -> Result<(), String> {
    app.run_on_main_thread(move || {
        #[cfg(target_os = "macos")]
        native_tauri_surface::surface_helper::native::pop_cursor();
    })
    .map_err(|e| format!("{e:?}"))
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn create_popout_window(
    app: tauri::AppHandle,
    label: String,
    url: String,
    title: String,
    width: f64,
    height: f64,
) -> Result<(), String> {
    tauri::WebviewWindowBuilder::new(&app, &label, tauri::WebviewUrl::App(url.into()))
        .title(title)
        .inner_size(width, height)
        .always_on_top(true)
        .disable_drag_drop_handler()
        .build()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            tauri::WebviewWindowBuilder::new(
                app,
                "main",
                tauri::WebviewUrl::App("index.html".into()),
            )
            .title("tauri-svelte")
            .inner_size(1200.0, 800.0)
            .disable_drag_drop_handler()
            .build()?;

            let surface_map: SurfaceMap = Arc::new(Mutex::new(HashMap::new()));
            app.manage(surface_map);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            create_popout_window,
            init_surface,
            render_surface,
            set_surface_rect,
            push_resize_cursor,
            pop_resize_cursor,
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app_handle, _event| {});
}
