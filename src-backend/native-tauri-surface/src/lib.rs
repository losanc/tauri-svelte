pub mod platform;
pub use platform::surface_context::CursorContext;

use std::sync::Arc;
mod log;
use wgpu::{Adapter, Device, Instance, Queue, Surface, SurfaceConfiguration, TextureFormat};

pub trait NativeSurfaceContext {
    fn hide_window(&self);
    fn current_window_size_and_position(&self) -> (u32, u32, u32, u32);
    fn move_window_size_and_position(&self, width: u32, height: u32, x: u32, y: u32);
}

pub trait WgpuSurfaceContext {
    fn get_wgpu_surface(&self) -> &Surface<'static>;

    fn get_configure(&self) -> SurfaceConfiguration {
        let surface = self.get_wgpu_surface();
        let configure = surface
            .get_configuration()
            .expect("failed to get current configure");
        configure
    }

    fn current_format(&self) -> TextureFormat {
        let configure = self.get_configure();
        configure.format
    }

    /// Returns the initial surface size in physical pixels `(width, height)`.
    fn current_render_size(&self) -> (u32, u32) {
        let configure = self.get_configure();
        (configure.width, configure.height)
    }

    fn change_render_resolution(
        &self,
        device: &Device,
        adapter: &Adapter,
        width: u32,
        height: u32,
    ) {
        let surface = self.get_wgpu_surface();
        let caps = surface.get_capabilities(adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);
        surface.configure(
            device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: format,
                width,
                height,
                present_mode: wgpu::PresentMode::Fifo,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            },
        );
    }
}

pub trait SurfaceContext: Send + Sync + WgpuSurfaceContext + NativeSurfaceContext {}

pub trait SurfaceSource {
    /// uses current window as parent window to create a subwindow
    fn create_child_surface(
        self,
        instance: &Instance,
        width: u32,
        height: u32,
        x: u32,
        y: u32,
    ) -> Arc<dyn SurfaceContext>;
}

/// Owns the wgpu device, queue, and surface for a single rendering target.
///
/// # Field Order — Drop Safety
///
/// `surface` is declared before `owner` intentionally. Rust drops fields top-to-bottom,
/// so the surface (which holds a raw pointer into `owner`'s native resources) is released
/// before the native resources themselves. **Do not reorder these fields.**
pub struct GpuContext {
    surfaces: Vec<Arc<dyn SurfaceContext>>, // drops 2nd — frees native resources (NSView / CAMetalLayer)
    device: Device,
    queue: Queue,
    adapter: Adapter,
    instance: Instance, // pub format: TextureFormat,
}

impl GpuContext {
    /// Initialize a wgpu instance from the provided surface source.
    ///
    /// Selects the best available adapter, requests a default device, picks an sRGB
    /// surface format when available, and configures the surface for rendering.
    /// Uses `PresentMode::Fifo` (vsync) and `CompositeAlphaMode::Auto`.
    #[must_use]
    pub async fn init() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::DX12,
            flags: wgpu::InstanceFlags::default(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            backend_options: wgpu::BackendOptions::default(),
            display: None,
        });
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .expect("create adapter failed");
        my_print!("device: {}", adapter.get_info().name);
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .expect("create device failed");

        Self {
            surfaces: Vec::new(),
            device,
            queue,
            adapter,
            instance,
        }
    }

    pub async fn add_surface(
        &mut self,
        surface_source: impl SurfaceSource,
        width: u32,
        height: u32,
        x: u32,
        y: u32,
    ) {
        let child_window = surface_source.create_child_surface(&self.instance, width, height, x, y);
        child_window.change_render_resolution(&self.device, &self.adapter, width, height);
        self.surfaces.push(child_window);
    }

    pub fn surfaces(&self) -> &[Arc<dyn SurfaceContext + 'static>] {
        &self.surfaces
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    pub fn adapter(&self) -> &Adapter {
        &self.adapter
    }

    pub fn instance(&self) -> &Instance {
        &self.instance
    }
}

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
