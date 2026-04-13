use crate::NativeSurfaceContext;
use crate::SurfaceContext;
use crate::SurfaceSource;
use crate::WgpuSurfaceContext;
use crate::platform::surface_context::CursorContext;
use std::sync::Arc;
use wgpu::Instance;
use wgpu::Surface;
use wgpu::SurfaceTarget;
use wgpu::web_sys::HtmlCanvasElement;

/// WASM GPU surface backed by an `HtmlCanvasElement`.
///
/// Wraps the canvas so it can be held as a [`SurfaceContext`] within [`GpuContext`](crate::GpuContext).
/// Geometry updates resize the canvas element directly; position is ignored because
/// the canvas is already laid out by the browser's DOM.
pub struct WasmContext {
    canvas: HtmlCanvasElement,
    surface: Surface<'static>,
}

impl WasmContext {
    /// Wrap an existing `HtmlCanvasElement`.
    pub fn new(canvas: HtmlCanvasElement, instance: &Instance) -> Self {
        let surface = instance
            .create_surface(SurfaceTarget::Canvas(canvas.clone()))
            .expect("create surface failed");
        let surface: wgpu::Surface<'static> = unsafe { std::mem::transmute(surface) };

        Self { canvas, surface }
    }
}

unsafe impl Send for WasmContext {}
unsafe impl Sync for WasmContext {}

impl SurfaceSource for HtmlCanvasElement {
    /// Create a wgpu WebGPU surface from the canvas and wrap it in a [`WasmContext`].
    ///
    /// The `Surface<'static>` lifetime is obtained via `transmute`. The canvas is
    /// cloned into `WasmContext`, so it remains alive for the surface's lifetime.
    fn create_child_surface(
        self,
        instance: &Instance,
        width: u32,
        height: u32,
        x: u32,
        y: u32,
    ) -> Box<dyn SurfaceContext> {
        Box::new(WasmContext::new(self, instance))
    }
}

impl WgpuSurfaceContext for WasmContext {
    fn get_wgpu_surface(&self) -> &wgpu::Surface<'static> {
        todo!()
    }
}
impl SurfaceContext for WasmContext {
    fn hash(&self) -> u64 {
        10
    }
}

impl NativeSurfaceContext for WasmContext {
    fn hide_window(&self) {
        todo!()
    }

    fn show_window(&self) {
        todo!()
    }

    fn current_window_size_and_position(&self) -> (u32, u32, u32, u32) {
        todo!()
    }

    fn move_window_size_and_position(&mut self, width: u32, height: u32, x: u32, y: u32) {
        todo!()
    }
    // /// Returns the canvas's intrinsic pixel size `(width, height)`.
    // fn initial_size(&self) -> (u32, u32) {
    //     (self.canvas.width(), self.canvas.height())
    // }

    // /// No-op on WASM — canvas visibility is controlled by the DOM / CSS.
    // fn hide(&self) {
    //     // Intentionally left empty: hiding is done via CSS (`display: none`) by
    //     // the SvelteKit panel layer, not by the Rust backend.
    // }

    // /// Resize the canvas to the new logical dimensions.
    // ///
    // /// `x`, `y`, and `window_height` are ignored — canvas position is determined
    // /// by normal DOM layout.
    // fn update_frame(&self, _x: f64, _y: f64, width: f64, height: f64, _window_height: f64) {
    //     self.canvas.set_width(width as u32);
    //     self.canvas.set_height(height as u32);
    // }

    // fn create_wgpu_surface(&self, instance: &wgpu::Instance) -> wgpu::Surface<'static> {
    //     // -> (WasmContext, wgpu::Surface<'static>) {
    //     let surface = instance
    //         .create_surface(SurfaceTarget::Canvas(self.canvas().clone()))
    //         .expect("create surface failed");
    //     let surface: wgpu::Surface<'static> = unsafe { std::mem::transmute(surface) };
    //     surface
    // }
}

impl CursorContext for WasmContext {
    /// No-op on WASM — cursor changes are handled via CSS `cursor` property.
    fn push_cursor(&self, _cursor: &str) {}

    /// No-op on WASM.
    fn pop_cursor(&self) {}
}
