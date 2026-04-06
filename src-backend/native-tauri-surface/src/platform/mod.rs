pub mod surface_context;

#[cfg(target_os = "macos")]
pub mod macos;
use std::sync::Arc;

#[cfg(target_os = "macos")]
pub use macos::{MacOSContext, pop_cursor, push_cursor};

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use windows::WindowsContext;

use crate::{SurfaceContext, SurfaceSource};

#[cfg(target_arch = "wasm32")]
pub mod web;

impl<T: raw_window_handle::HasWindowHandle> SurfaceSource for T {
    fn create_surface_context(
        self,
        instance: &wgpu::Instance,
        width: u32,
        height: u32,
        x: u32,
        y: u32,
    ) -> Arc<dyn SurfaceContext> {
        let raw_handle = self.window_handle().unwrap();
        match raw_handle.as_raw() {
            #[cfg(target_os = "macos")]
            raw_window_handle::RawWindowHandle::AppKit(app_kit_window_handle) => {
                let ctx: MacOSContext = MacOSContext::new(&self, 1, 1, 0, 0);
                Arc::new(ctx)
            }
            #[cfg(target_os = "windows")]
            raw_window_handle::RawWindowHandle::Win32(win32_window_handle) => {
                use ::windows::Win32::System::LibraryLoader::GetModuleHandleW;
                use std::sync::Arc;
                let windows_context =
                    WindowsContext::new(win32_window_handle.hwnd, width, height, x, y);
                Arc::new(windows_context)
            }
            _ => {
                panic!("unsupported platform");
            }
        }
    }
}
