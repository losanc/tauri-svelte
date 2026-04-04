use crate::platform::surface_context::{CursorContext, SurfaceContext, SurfaceSource};
use wgpu::SurfaceTarget;
use wgpu::web_sys::HtmlCanvasElement;

pub struct WasmContext {
    canvas: HtmlCanvasElement,
}

impl WasmContext {
    pub fn new(canvas: HtmlCanvasElement) -> Self {
        Self { canvas }
    }
    pub fn canvas(&self) -> &HtmlCanvasElement {
        &self.canvas
    }
}

impl SurfaceSource for HtmlCanvasElement {
    type Context = WasmContext;
    fn create(self, instance: &wgpu::Instance) -> (WasmContext, wgpu::Surface<'static>) {
        let surface = instance
            .create_surface(SurfaceTarget::Canvas(self.clone()))
            .expect("create surface failed");
        let surface: wgpu::Surface<'static> = unsafe { std::mem::transmute(surface) };
        (WasmContext::new(self), surface)
    }
}

impl SurfaceContext for WasmContext {
    fn initial_size(&self) -> (u32, u32) {
        (self.canvas.width(), self.canvas.height())
    }

    fn hide(&self) {
        // self.canvas.style().set_property("display", "none").ok();
    }

    fn update_frame(&self, _x: f64, _y: f64, width: f64, height: f64, _window_height: f64) {
        self.canvas.set_width(width as u32);
        self.canvas.set_height(height as u32);
    }
}

impl CursorContext for WasmContext {
    fn push_cursor(&self, _cursor: &str) {}
    fn pop_cursor(&self) {}
}
