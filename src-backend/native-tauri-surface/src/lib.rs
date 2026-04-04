pub mod surface_helper;
pub use surface_helper::{SurfaceOwner, SurfaceSource};

#[cfg(not(target_arch = "wasm32"))]
pub use surface_helper::native::SurfaceResizer;

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
    surface: Surface<'static>, // drops 1st — releases raw pointer
    owner: SurfaceOwner,       // drops 2nd — frees native resources (NSView / CAMetalLayer)
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
        let (owner, surface, width, height) = source.create(&instance);
        // SAFETY: `owner` is stored in this struct alongside `surface`.
        // The field declaration order guarantees `surface` drops before `owner`,
        // preserving the borrow for the entire lifetime of `GpuContext`.
        // Both `create_surface_unsafe` (native) and `SurfaceTarget::Canvas` (WASM)
        // already return `Surface<'static>` from wgpu; the transmute is a no-op in
        // practice but required because the trait expresses `Surface<'_>`.
        let surface: Surface<'static> = unsafe { std::mem::transmute(surface) };

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
                width,
                height,
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

    /// Returns a [`SurfaceResizer`] — a lightweight clone of the native handles
    /// used to reposition and resize the Metal layer from the main thread.
    ///
    /// Only available on native (non-WASM) targets.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn resizer(&self) -> SurfaceResizer {
        self.owner.resizer()
    }

    /// Returns the `HtmlCanvasElement` that backs this surface.
    ///
    /// Only available on WASM targets.
    #[cfg(target_arch = "wasm32")]
    pub fn canvas(&self) -> &wgpu::web_sys::HtmlCanvasElement {
        self.owner.canvas()
    }
}
