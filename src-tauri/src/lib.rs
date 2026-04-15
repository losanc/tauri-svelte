use native_tauri_surface::SurfaceHash;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use tauri::{Manager, State};
use utils_api::fs;
use wgpu_renderer::Renderer;

type SurfaceMap = Mutex<Option<HashMap<String, HashSet<SurfaceHash>>>>;
type ThreadSafeRenderer = Mutex<Option<Renderer>>;

fn browser_pixel_to_native_pixel(
    current_window: &tauri::WebviewWindow,
    width: f64,
    height: f64,
    x: f64,
    y: f64,
) -> Result<(u32, u32, u32, u32), String> {
    #[cfg(target_os = "macos")]
    {
        let _ = current_window;
        return Ok((width as u32, height as u32, x as u32, y as u32));
    }
    #[cfg(not(target_os = "macos"))]
    {
        let scale = current_window.scale_factor().map_err(|e| e.to_string())?;
        return Ok((
            (width * scale) as u32,
            (height * scale) as u32,
            (x * scale) as u32,
            (y * scale) as u32,
        ));
    }
}

/// use browser resolution. width height can be float. e.g. windows set to 200% zoom. it can have .5 resolution from html
async fn add_wgpu_sub_surface_impl(
    current_window: tauri::WebviewWindow,
    renderer: &mut Renderer,
    width: f64,
    height: f64,
    x: f64,
    y: f64,
) -> Result<SurfaceHash, String> {
    let (width, height, x, y) =
        browser_pixel_to_native_pixel(&current_window, width, height, x, y)?;
    let hash = renderer
        .add_surface(current_window, width, height, x, y)
        .await;
    Ok(hash)
}

#[tauri::command]
fn add_wgpu_native_surface(
    current_window: tauri::WebviewWindow,
    renderer: tauri::State<'_, ThreadSafeRenderer>,
    surface_map: tauri::State<'_, SurfaceMap>,
    width: f64,
    height: f64,
    x: f64,
    y: f64,
) -> Result<String, String> {
    println!("rust gives ");

    let label = current_window.label().to_string();

    let hash = pollster::block_on(add_wgpu_sub_surface_impl(
        current_window,
        renderer
            .lock()
            .expect("should have value")
            .as_mut()
            .expect("wrong"),
        width,
        height,
        x,
        y,
    ))
    .expect("falied");
    println!("rust gives {}", hash);

    if let Some(surface_map) = surface_map.lock().expect("how").as_mut() {
        match surface_map.get_mut(&label) {
            Some(list) => {
                list.insert(hash);
            }
            None => {
                surface_map.insert(label, HashSet::from([hash]));
            }
        }
    };

    Ok(hash.to_string())
}

#[tauri::command]
fn display_wgpu_native_surface(renderer: tauri::State<'_, ThreadSafeRenderer>, hash: String) {
    let renderer = renderer.lock().expect("should have value");
    let renderer_inner = renderer.as_ref().unwrap();
    // .as_mut().expect("wrong");
    renderer_inner.show_surface(hash.into());
}

#[tauri::command]
fn hide_wgpu_native_surface(renderer: tauri::State<'_, ThreadSafeRenderer>, hash: String) {
    let renderer = renderer.lock().expect("should have value");
    let renderer_inner = renderer.as_ref().unwrap();
    // .as_mut().expect("wrong");
    renderer_inner.hide_surface(hash.into());
}
#[tauri::command]
fn destroy_wgpu_native_surface(
    renderer: tauri::State<'_, ThreadSafeRenderer>,
    current_window: tauri::WebviewWindow,
    surface_map: tauri::State<'_, SurfaceMap>,
    hash: String,
) {
    let mut renderer = renderer.lock().expect("should have value");
    let renderer_inner = renderer.as_mut().unwrap();
    let hash = hash.into();
    renderer_inner.destroy_surface(hash);
    let label = current_window.label().to_string();

    if let Some(surface_map) = surface_map.lock().expect("how").as_mut() {
        match surface_map.get_mut(&label) {
            Some(list) => {
                list.remove(&hash);
            }
            None => {
                panic!("what the hell");
            }
        }
    };
}

#[tauri::command]
fn move_wgpu_native_surface(
    renderer: tauri::State<'_, ThreadSafeRenderer>,
    current_window: tauri::WebviewWindow,
    hash: String,
    width: f64,
    height: f64,
    x: f64,
    y: f64,
) -> Result<(), String> {
    println!("moveed to new location {width} {height} {x} {y}");
    let mut renderer = renderer.lock().expect("should have value");
    let renderer_inner = renderer.as_mut().unwrap();
    let (width, height, x, y) =
        browser_pixel_to_native_pixel(&current_window, width, height, x, y)?;
    renderer_inner.set_surface_position(hash.into(), width, height, x, y);
    Ok(())
}

// fn set_surface_rect_impl(
//     app: &tauri::AppHandle,
//     window: tauri::WebviewWindow,
//     surfaces: tauri::State<'_, SurfaceMap>,
//     x: f64,
//     y: f64,
//     width: f64,
//     height: f64,
// ) -> Result<(), String> {
//     let scale = window.scale_factor().map_err(|e| e.to_string())?;
//     let inner_size = window.inner_size().map_err(|e| e.to_string())?;
//     let window_height = inner_size.height as f64 / scale;

//     let renderer = {
//         let map = surfaces.lock().unwrap();
//         map.get(window.label()).expect("no renderer?");
//     };

//     if width <= 0.0 || height <= 0.0 {
//         return app
//             .run_on_main_thread(move || renderer.hide())
//             .map_err(|e| format!("{e:?}"));
//     }

//     // GPU resize outside the lock — surface reconfigure must not block other commands.
//     renderer.resize((width * scale) as u32, (height * scale) as u32);

//     app.run_on_main_thread(move || {
//         renderer.update_frame(x, y, width, height, window_height);
//     })
//     .map_err(|e| format!("{e:?}"))
// }

/// Create a native wgpu surface for the calling window.
/// Idempotent — safe to call even if the surface already exists.
// #[tauri::command]
// fn init_surface(
//     app: tauri::AppHandle,
//     window: tauri::WebviewWindow,
//     surfaces: tauri::State<'_, SurfaceMap>,
// ) -> Result<(), String> {
//     add_wgpu_sub_surface_impl(&app, window, surfaces.inner())
// }

// #[tauri::command]
// fn set_surface_rect(
//     app: tauri::AppHandle,
//     window: tauri::WebviewWindow,
//     surfaces: tauri::State<'_, SurfaceMap>,
//     x: f64,
//     y: f64,
//     width: f64,
//     height: f64,
// ) -> Result<(), String> {
// }

// #[tauri::command]
// fn render_surface(window: tauri::WebviewWindow, surfaces: tauri::State<'_, SurfaceMap>) {
//     return ();

//     let renderer = surfaces
//         .lock()
//         .ok()
//         .and_then(|map| map.get(window.label()).map(Arc::clone));
//     if let Some(renderer) = renderer {
//         renderer.render();
//     }
// }

/// Push a resize cursor onto the macOS cursor stack.
/// Pushed cursors take precedence over NSWindow cursor rects (including
/// WKWebView's), so this reliably shows the resize icon regardless of what
/// WKWebView thinks the CSS cursor should be.
#[tauri::command]
fn push_resize_cursor(app: tauri::AppHandle, horizontal: bool) -> Result<(), String> {
    app.run_on_main_thread(move || {
        // #[cfg(target_os = "macos")]
        // native_tauri_surface::push_cursor(if horizontal { "ew-resize" } else { "ns-resize" });
    })
    .map_err(|e| format!("{e:?}"))
}

/// Pop the top cursor from the macOS cursor stack.
#[tauri::command]
fn pop_resize_cursor(app: tauri::AppHandle) -> Result<(), String> {
    app.run_on_main_thread(move || {
        // #[cfg(target_os = "macos")]
        // native_tauri_surface::pop_cursor();
    })
    .map_err(|e| format!("{e:?}"))
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn create_popout_window(
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
            let handle = app.handle();
            let window = tauri::WebviewWindowBuilder::new(
                handle,
                "main",
                tauri::WebviewUrl::App("index.html".into()),
            )
            .title("tauri-svelte")
            .inner_size(1200.0, 800.0)
            .disable_drag_drop_handler()
            .build()?;
            let handle_copy = handle.clone();
            let label = window.label().to_string();
            let label_clone = label.clone();
            window.on_window_event(move |event| match event {
                tauri::WindowEvent::Destroyed => {
                    let state: State<SurfaceMap> = handle_copy.state();
                    let renderer: State<ThreadSafeRenderer> = handle_copy.state();
                    let mut renderer = renderer.lock().expect("should have value");

                    let mut surface_map = state.lock().expect("lock failed");
                    let surface_map = surface_map.as_mut().expect("can't be none");
                    let hashes = surface_map.remove(&label).expect("can't be none");
                    for hash in hashes {
                        let renderer_inner = renderer.as_mut().unwrap();
                        renderer_inner.destroy_surface(hash);
                    }
                    println!("window destroied");
                }
                _ => {}
            });

            let renderer: ThreadSafeRenderer =
                Mutex::new(Some(pollster::block_on(Renderer::new())));
            let surface_map: SurfaceMap =
                Mutex::new(Some(HashMap::from([(label_clone, HashSet::new())])));
            app.manage(renderer);
            app.manage(surface_map);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            create_popout_window,
            // init_surface,
            // render_surface,
            // set_surface_rect,
            move_wgpu_native_surface,
            destroy_wgpu_native_surface,
            add_wgpu_native_surface,
            display_wgpu_native_surface,
            hide_wgpu_native_surface,
            push_resize_cursor,
            pop_resize_cursor,
            list_directory,
            get_home_dir,
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app_handle, event| match event {
            tauri::RunEvent::Exit => {
                // manually drop the state
                // see https://github.com/tauri-apps/tauri/issues/14420
                {
                    let state: State<ThreadSafeRenderer> = app_handle.state();
                    let renderer = state.lock().expect("well").take();
                    std::mem::drop(renderer);
                }
                {
                    let state: State<SurfaceMap> = app_handle.state();
                    let surface_map = state.lock().expect("well").take();
                    // surface_map should be empty. it should be destructed when closing window
                    debug_assert!(surface_map.clone().unwrap().len() == 0);
                    std::mem::drop(surface_map);
                }
                println!("close");
            }
            _ => {}
        });
}
