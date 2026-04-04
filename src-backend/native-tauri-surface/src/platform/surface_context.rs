pub trait SurfaceContext: Send + Sync {
    /// Returns the initial surface size in physical pixels `(width, height)`.
    fn initial_size(&self) -> (u32, u32);

    /// Hide the surface.
    ///
    /// Must be called on the main thread.
    fn hide(&self);

    /// Reposition and resize the surface within its parent window.
    ///
    /// - `x`, `y` — panel position in logical (CSS) pixels, top-left origin.
    /// - `width`, `height` — panel size in logical pixels.
    /// - `window_height` — logical height of the window's inner content area.
    ///
    /// Must be called on the main thread.
    fn update_frame(&self, x: f64, y: f64, width: f64, height: f64, window_height: f64);
}

/// Consumes a platform resource to produce a [`SurfaceContext`] and a wgpu [`Surface`](wgpu::Surface).
///
/// The returned `Surface<'static>` is obtained via an `unsafe` lifetime transmute;
/// the [`GpuContext`](crate::GpuContext) field ordering guarantees the surface is
/// dropped before the owning context.
pub trait SurfaceSource {
    type Context: SurfaceContext + 'static;

    /// Create a [`SurfaceContext`] and a wgpu surface from this source.
    fn create(self, instance: &wgpu::Instance) -> (Self::Context, wgpu::Surface<'static>);
}

pub trait CursorContext {
    /// Push a named cursor onto the cursor stack.
    fn push_cursor(&self, cursor: &str);

    /// Pop the top cursor from the cursor stack.
    /// Must be paired with a prior [`push_cursor`](Self::push_cursor) call.
    fn pop_cursor(&self);
}
