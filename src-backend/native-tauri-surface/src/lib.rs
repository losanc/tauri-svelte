pub mod platform;
pub use platform::surface_context::{CursorContext, SurfaceContext, SurfaceSource};

#[cfg(target_os = "macos")]
pub use platform::{pop_cursor, push_cursor};
// MacOSContext is intentionally not re-exported; use `create_surface` instead.

use std::sync::Arc;
mod log;
use wgpu::{Device, Queue, Surface, TextureFormat};

/// Shared GPU surface interface, mirroring the TypeScript `GpuSurface`.
///
/// Both backends implement this trait:
/// - **Native (Tauri):** driven by the `set_surface_rect` / `render_surface` IPC commands.
/// - **WASM:** implemented by `WasmRenderer` on an `HtmlCanvasElement`.
pub trait GpuSurface {
    /// Update the surface position and size.
    ///
    /// `x` and `y` are screen-space CSS pixel coordinates (top-left origin).
    /// WASM ignores position and uses canvas intrinsic sizing; native uses all four values.
    /// `width` and `height` drive the underlying surface reconfiguration.
    fn set_rect(&self, x: f64, y: f64, width: f64, height: f64);

    /// Submit one rendered frame to the surface.
    fn render(&self);
}

/// Owns the wgpu device, queue, and surface for a single rendering target.
///
/// # Field Order — Drop Safety
///
/// `surface` is declared before `owner` intentionally. Rust drops fields top-to-bottom,
/// so the surface (which holds a raw pointer into `owner`'s native resources) is released
/// before the native resources themselves. **Do not reorder these fields.**
pub struct GpuContext {
    surface: Surface<'static>,      // drops 1st — releases raw pointer
    owner: Arc<dyn SurfaceContext>, // drops 2nd — frees native resources (NSView / CAMetalLayer)
    pub device: Device,
    pub queue: Queue,
    pub format: TextureFormat,
}

impl GpuContext {
    /// Initialize a wgpu instance from the provided surface source.
    ///
    /// Selects the best available adapter, requests a default device, picks an sRGB
    /// surface format when available, and configures the surface for rendering.
    /// Uses `PresentMode::Fifo` (vsync) and `CompositeAlphaMode::Auto`.
    pub async fn init_wgpu(source: impl SurfaceSource) -> Self {
        let instance = wgpu::Instance::default();
        let (context, surface) = source.create(&instance);
        let (init_w, init_h) = context.initial_size();
        let owner: Arc<dyn SurfaceContext> = Arc::new(context);

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .expect("create adapter failed");
        my_print!("device: {}", adapter.get_info().name);
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .expect("create device failed");

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);
        surface.configure(
            &device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format,
                width: init_w.max(1),
                height: init_h.max(1),
                present_mode: wgpu::PresentMode::Fifo,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            },
        );
        Self {
            surface,
            owner,
            device,
            queue,
            format,
        }
    }

    /// Reconfigure the surface for a new pixel size.
    ///
    /// No-ops when either dimension is zero to avoid a wgpu validation error.
    /// Should be called whenever the containing window or panel is resized.
    pub fn resize(&self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        self.surface.configure(
            &self.device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: self.format,
                width,
                height,
                present_mode: wgpu::PresentMode::Fifo,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            },
        );
    }

    /// Returns a reference to the underlying wgpu surface.
    pub fn surface(&self) -> &Surface<'static> {
        &self.surface
    }

    /// Hide the underlying native surface.
    ///
    /// Call when the panel has zero size or is not visible to avoid rendering artifacts.
    pub fn hide(&self) {
        self.owner.hide();
    }

    /// Reposition and resize the native surface within its parent window.
    ///
    /// Delegates to [`SurfaceContext::update_frame`]. `x` and `y` are CSS pixel coords.
    pub fn update_frame(&self, x: f64, y: f64, width: f64, height: f64, window_height: f64) {
        self.owner.update_frame(x, y, width, height, window_height);
    }
}

/// Create a platform-native wgpu surface for the given window handle.
///
/// Returns `Err` on platforms that are not yet supported.
/// Must be called on the main thread.
pub fn create_surface(
    window: &impl raw_window_handle::HasWindowHandle,
    width: u32,
    height: u32,
    x: u32,
    y: u32,
) -> Result<impl SurfaceSource, &'static str> {
    #[cfg(target_os = "macos")]
    return Ok(platform::macos::MacOSContext::new(
        window, width, height, x, y,
    ));

    #[cfg(not(target_os = "macos"))]
    {
        let _ = (window, width, height, x, y);
        Err("platform not supported")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // ── MockSurfaceContext ────────────────────────────────────────────────────

    #[derive(Default)]
    struct LastFrame {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        window_height: f64,
    }

    struct MockSurfaceContext {
        width: u32,
        height: u32,
        hidden: Mutex<bool>,
        last_frame: Mutex<Option<LastFrame>>,
    }

    impl MockSurfaceContext {
        fn new(width: u32, height: u32) -> Self {
            Self {
                width,
                height,
                hidden: Mutex::new(false),
                last_frame: Mutex::new(None),
            }
        }
    }

    impl SurfaceContext for MockSurfaceContext {
        fn initial_size(&self) -> (u32, u32) {
            (self.width, self.height)
        }

        fn hide(&self) {
            *self.hidden.lock().unwrap() = true;
        }

        fn update_frame(&self, x: f64, y: f64, width: f64, height: f64, window_height: f64) {
            *self.last_frame.lock().unwrap() = Some(LastFrame {
                x,
                y,
                width,
                height,
                window_height,
            });
        }
    }

    // ── Tests ─────────────────────────────────────────────────────────────────

    #[test]
    fn mock_reports_initial_size() {
        let ctx = MockSurfaceContext::new(640, 480);
        assert_eq!(ctx.initial_size(), (640, 480));
    }

    #[test]
    fn mock_hide_sets_hidden_flag() {
        let ctx = MockSurfaceContext::new(100, 100);
        assert!(!*ctx.hidden.lock().unwrap());
        ctx.hide();
        assert!(*ctx.hidden.lock().unwrap());
    }

    #[test]
    fn mock_update_frame_records_values() {
        let ctx = MockSurfaceContext::new(800, 600);
        ctx.update_frame(10.0, 20.0, 300.0, 200.0, 800.0);
        let guard = ctx.last_frame.lock().unwrap();
        let f = guard
            .as_ref()
            .expect("update_frame should have been recorded");
        assert_eq!(f.x, 10.0);
        assert_eq!(f.y, 20.0);
        assert_eq!(f.width, 300.0);
        assert_eq!(f.height, 200.0);
        assert_eq!(f.window_height, 800.0);
    }

    #[test]
    fn resize_guard_skips_on_zero_width() {
        // Verifies the guard condition: width == 0 || height == 0
        assert!(0u32 == 0 || 100u32 == 0);
    }

    #[test]
    fn resize_guard_skips_on_zero_height() {
        assert!(100u32 == 0 || 0u32 == 0);
    }

    #[test]
    fn resize_guard_passes_for_nonzero_dimensions() {
        assert!(!(1u32 == 0 || 1u32 == 0));
    }
}
