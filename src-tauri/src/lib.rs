use native_tauri_surface::{SurfaceResizer, surface_helper::native::SurfaceHelper};
use wgpu_renderer::Renderer;
use tauri::Manager;

#[tauri::command]
fn set_surface_rect(
    app: tauri::AppHandle,
    resizer: tauri::State<'_, SurfaceResizer>,
    renderer: tauri::State<'_, Renderer>,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), String> {
    if width <= 0.0 || height <= 0.0 { return Ok(()); }
    let window = app.get_webview_window("main").ok_or("no window")?;
    let scale = window.scale_factor().map_err(|e| e.to_string())?;
    let inner_size = window.inner_size().map_err(|e| e.to_string())?;
    let window_height = inner_size.height as f64 / scale;

    renderer.resize((width * scale) as u32, (height * scale) as u32);

    let ptr = resizer.ns_view_ptr;
    app.run_on_main_thread(move || unsafe {
        SurfaceResizer { ns_view_ptr: ptr }.update_frame(x, y, width, height, window_height);
    })
    .map_err(|e| format!("{e:?}"))
}

#[tauri::command]
fn render_surface(state: tauri::State<'_, Renderer>) {
    state.render();
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
            let window = tauri::WebviewWindowBuilder::new(
                app,
                "main",
                tauri::WebviewUrl::App("index.html".into()),
            )
            .title("tauri-svelte")
            .inner_size(1200.0, 800.0)
            .disable_drag_drop_handler()
            .build()?;

            let (tauri_surface, surface_resizer) = SurfaceHelper::new(&window, 200, 200, 20, 20);
            let renderer = pollster::block_on(Renderer::new(tauri_surface));
            app.manage(renderer);
            app.manage(surface_resizer);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            create_popout_window,
            render_surface,
            set_surface_rect
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app_handle, _event| {});
}
