pub trait SurfaceContext: Send + Sync {
    fn initial_size(&self) -> (u32, u32);
    fn hide(&self);
    fn update_frame(&self, x: f64, y: f64, width: f64, height: f64, window_height: f64);
}

pub trait SurfaceSource {
    type Context: SurfaceContext + 'static;
    fn create(self, instance: &wgpu::Instance) -> (Self::Context, wgpu::Surface<'static>);
}

pub trait CursorContext {
    fn push_cursor(&self, cursor: &str);
    fn pop_cursor(&self);
}
