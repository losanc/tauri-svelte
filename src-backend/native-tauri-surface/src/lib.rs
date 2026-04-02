pub mod surface_helper;
pub use surface_helper::{SurfaceOwner, SurfaceSource};

#[cfg(not(target_arch = "wasm32"))]
pub use surface_helper::native::SurfaceResizer;

mod log;
use wgpu::{Device, Queue, Surface, TextureFormat};

/// Shared surface interface, mirroring the TypeScript `GpuSurface`.
/// - Native (Tauri): driven by the `set_surface_rect` / `render_surface` commands.
/// - WASM: implemented by `WasmRenderer`.
pub trait GpuSurface {
    /// CSS-pixel rect. `x`/`y` are screen position (ignored by WASM); width/height drive resize.
    fn set_rect(&self, x: f64, y: f64, width: f64, height: f64);
    fn render(&self);
}

pub struct GpuContext {
    // FIELD ORDER IS LOAD-BEARING.
    // Rust drops fields top-to-bottom. `surface` must drop before `owner` because
    // `surface` holds a raw pointer into data owned by `owner` (e.g. CAMetalLayer).
    surface: Surface<'static>, // drops 1st
    owner: SurfaceOwner,       // drops 2nd — keeps native resources alive
    pub device: Device,
    pub queue: Queue,
    pub format: TextureFormat,
}

impl GpuContext {
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

    pub fn surface(&self) -> &Surface<'static> {
        &self.surface
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn resizer(&self) -> SurfaceResizer {
        self.owner.resizer()
    }

    #[cfg(target_arch = "wasm32")]
    pub fn canvas(&self) -> &wgpu::web_sys::HtmlCanvasElement {
        self.owner.canvas()
    }
}
