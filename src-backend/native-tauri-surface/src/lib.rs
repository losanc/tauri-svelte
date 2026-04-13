pub mod platform;
pub use platform::surface_context::CursorContext;

use std::{collections::HashMap, sync::Arc};
mod log;
use wgpu::{
    Adapter, Device, Instance, Queue, Surface, SurfaceCapabilities, SurfaceConfiguration,
    TextureFormat,
};

pub trait NativeSurfaceContext {
    fn hide_window(&self);
    fn show_window(&self);
    fn current_window_size_and_position(&self) -> (u32, u32, u32, u32);
    fn move_window_size_and_position(&mut self, width: u32, height: u32, x: u32, y: u32);
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
    fn get_capabilities(&self, adapter: &Adapter) -> SurfaceCapabilities {
        let surface = self.get_wgpu_surface();
        surface.get_capabilities(adapter)
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

    fn set_render_resolution(&self, device: &Device, adapter: &Adapter, width: u32, height: u32) {
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

pub trait SurfaceContext: Send + Sync + WgpuSurfaceContext + NativeSurfaceContext {
    fn hash(&self) -> u64;
}

pub trait SurfaceSource {
    /// uses current window as parent window to create a subwindow
    fn create_child_surface(
        self,
        instance: &Instance,
        width: u32,
        height: u32,
        x: u32,
        y: u32,
    ) -> Box<dyn SurfaceContext>;
}

/// Owns the wgpu device, queue, and surface for a single rendering target.
///
/// # Field Order — Drop Safety
///
/// `surface` is declared before `owner` intentionally. Rust drops fields top-to-bottom,
/// so the surface (which holds a raw pointer into `owner`'s native resources) is released
/// before the native resources themselves. **Do not reorder these fields.**
pub struct GpuContext {
    surfaces: HashMap<u64, Box<dyn SurfaceContext>>, // drops 2nd — frees native resources (NSView / CAMetalLayer)
    queue: Queue,
    device: Device,
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
            surfaces: HashMap::new(),
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
    ) -> u64 {
        let child_window = surface_source.create_child_surface(&self.instance, width, height, x, y);
        child_window.set_render_resolution(&self.device, &self.adapter, width, height);
        let hash = child_window.hash();
        self.surfaces.insert(hash, child_window);
        hash
    }

    pub fn move_surface(&mut self, hash: u64, width: u32, height: u32, x: u32, y: u32) {
        if let Some(window) = self.surfaces.get_mut(&hash) {
            window
                .as_mut()
                .move_window_size_and_position(width, height, x, y);
            window
                .as_mut()
                .set_render_resolution(&self.device, &self.adapter, width, height);
        }
    }

    pub fn remove_surface(&mut self, hash: u64) {
        println!("destroied surface {hash}");
        let _ = self.surfaces.remove(&hash);
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

    pub fn surfaces(&self) -> &HashMap<u64, Box<dyn SurfaceContext>> {
        &self.surfaces
    }
}
