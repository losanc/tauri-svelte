use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::Manager;
use utils_api::fs;
use wgpu_renderer::Renderer;

pub type SurfaceMap = Arc<Mutex<HashMap<String, Arc<Renderer>>>>;

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

    #[cfg(target_os = "macos")]
    {
        app.run_on_main_thread(move || {
            let tauri_surface =
                native_tauri_surface::create_surface(&window, 1, 1, 0, 0).unwrap();
            let renderer = Arc::new(pollster::block_on(Renderer::new(tauri_surface)));
            map.lock().unwrap().insert(label, renderer);
        })
        .map_err(|e| format!("{e:?}"))?;
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = (app, window, map, label);
        return Err("platform not supported".into());
    }

    Ok(())
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
    let scale = window.scale_factor().map_err(|e| e.to_string())?;
    let inner_size = window.inner_size().map_err(|e| e.to_string())?;
    let window_height = inner_size.height as f64 / scale;

    let renderer = {
        let map = surfaces.lock().unwrap();
        map.get(window.label())
            .map(Arc::clone)
            .ok_or("surface not initialized")?
    };

    if width <= 0.0 || height <= 0.0 {
        return app
            .run_on_main_thread(move || renderer.hide())
            .map_err(|e| format!("{e:?}"));
    }

    // GPU resize outside the lock — surface reconfigure must not block other commands.
    renderer.resize((width * scale) as u32, (height * scale) as u32);

    app.run_on_main_thread(move || {
        renderer.update_frame(x, y, width, height, window_height);
    })
    .map_err(|e| format!("{e:?}"))
}

#[tauri::command]
fn render_surface(window: tauri::WebviewWindow, surfaces: tauri::State<'_, SurfaceMap>) {
    let renderer = surfaces
        .lock()
        .ok()
        .and_then(|map| map.get(window.label()).map(Arc::clone));
    if let Some(renderer) = renderer {
        renderer.render();
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
        native_tauri_surface::push_cursor(if horizontal { "ew-resize" } else { "ns-resize" });
    })
    .map_err(|e| format!("{e:?}"))
}

/// Pop the top cursor from the macOS cursor stack.
#[tauri::command]
fn pop_resize_cursor(app: tauri::AppHandle) -> Result<(), String> {
    app.run_on_main_thread(move || {
        #[cfg(target_os = "macos")]
        native_tauri_surface::pop_cursor();
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

#[tauri::command]
fn list_directory(path: String) -> Result<Vec<fs::DirEntry>, String> {
    fs::list_dirs(&path)
}

#[tauri::command]
fn get_home_dir() -> Result<String, String> {
    fs::get_home()
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
            list_directory,
            get_home_dir,
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app_handle, _event| {});
}
