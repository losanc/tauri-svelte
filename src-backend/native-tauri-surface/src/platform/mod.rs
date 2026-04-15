pub mod surface_context;

#[cfg(target_os = "macos")]
pub mod macos;
use std::sync::Arc;

#[cfg(target_os = "macos")]
pub use macos::{MacOSContext, pop_cursor, push_cursor};

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(not(target_arch = "wasm32"))]
use wgpu::Instance;
#[cfg(target_os = "windows")]
pub use windows::WindowsContext;

use crate::{SurfaceContext, SurfaceSource};

#[cfg(target_arch = "wasm32")]
pub mod web;

#[cfg(not(target_arch = "wasm32"))]
impl<T: raw_window_handle::HasWindowHandle> SurfaceSource for T {
    fn create_child_surface(
        self,
        instance: &Instance,
        width: u32,
        height: u32,
        x: u32,
        y: u32,
    ) -> Box<dyn SurfaceContext> {
        let raw_handle = self.window_handle().unwrap();
        match raw_handle.as_raw() {
            #[cfg(target_os = "macos")]
            raw_window_handle::RawWindowHandle::AppKit(_app_kit_window_handle) => {
                Box::new(MacOSContext::new(instance, &self, width, height, x, y))
            }
            #[cfg(target_os = "windows")]
            raw_window_handle::RawWindowHandle::Win32(win32_window_handle) => Box::new(
                WindowsContext::new(instance, win32_window_handle.hwnd, width, height, x, y),
            ),
            _ => {
                panic!("unsupported platform");
            }
        }
    }
}
