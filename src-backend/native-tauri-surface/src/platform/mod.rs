pub mod surface_context;

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
pub use macos::{MacOSContext, pop_cursor, push_cursor};

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use windows::WindowsContext;

#[cfg(target_arch = "wasm32")]
pub mod web;
