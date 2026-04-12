use crate::platform::surface_context::CursorContext;
use crate::{NativeSurfaceContext, SurfaceContext, WgpuSurfaceContext};
use raw_window_handle::{RawWindowHandle, Win32WindowHandle, WindowsDisplayHandle};
use std::num::NonZeroIsize;
use wgpu::Instance;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    BringWindowToTop, CreateWindowExW, HWND_TOP, SW_HIDE, SW_SHOW, SWP_NOACTIVATE, SWP_NOZORDER,
    SetWindowPos, ShowWindow, WINDOW_STYLE, WS_CHILD, WS_VISIBLE,
};

/// Windows GPU surface backed by a Win32 child `HWND`.
///
/// A borderless child window is created inside the Tauri window's `HWND` and
/// handed to wgpu as a `Win32` surface target (D3D12 / Vulkan backend).
///
/// `Send + Sync` are asserted manually because `HWND` is a raw pointer wrapper.
/// All Win32 UI calls must happen on the main thread.
pub struct WindowsContext {
    hwnd: HWND,
    width: u32,
    height: u32,
    x: u32,
    y: u32,
    wgpu_surface: wgpu::Surface<'static>,
}

unsafe impl Send for WindowsContext {}
unsafe impl Sync for WindowsContext {}

impl WindowsContext {
    /// Create a borderless child `HWND` inside the given Tauri window.
    ///
    /// # Parameters
    /// - `window` — any type providing a raw window handle (e.g. `tauri::WebviewWindow`).
    /// - `width`, `height` — initial surface size in physical pixels.
    /// - `x`, `y` — initial position within the parent window in physical pixels.
    ///
    /// # Panics
    /// Panics if the window handle is not a Win32 handle or if `CreateWindowExW` fails.
    pub fn new(
        instance: &Instance,
        hwnd: NonZeroIsize,
        width: u32,
        height: u32,
        x: u32,
        y: u32,
    ) -> Self {
        let parent_hwnd = HWND(hwnd.get() as *mut std::ffi::c_void);

        // A null class name re-uses the parent's WNDCLASS which is sufficient for
        // a child surface that only serves as a wgpu render target.
        let child_hwnd = unsafe {
            CreateWindowExW(
                Default::default(),                      // dwExStyle
                windows::core::w!("STATIC"), // lpClassName — built-in, always registered
                Option::None,                // lpWindowName
                WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0), // dwStyle
                x as i32,                    // X
                y as i32,                    // Y
                width as i32,                // nWidth
                height as i32,               // nHeight
                Some(parent_hwnd),           // hWndParent
                None,                        // hMenu
                None,                        // hInstance (None = current module)
                None,                        // lpParam
            )
        }
        .expect("CreateWindowExW failed");

        unsafe { BringWindowToTop(child_hwnd).expect("falied to bring window to top") };

        let target: wgpu::SurfaceTargetUnsafe = wgpu::SurfaceTargetUnsafe::RawHandle {
            raw_display_handle: Some(raw_window_handle::RawDisplayHandle::Windows(
                WindowsDisplayHandle::new(),
            )),
            raw_window_handle: RawWindowHandle::Win32(Win32WindowHandle::new(
                NonZeroIsize::new(child_hwnd.0 as isize).expect("windows handle is nullptr"),
            )),
        };
        let surface = unsafe {
            instance
                .create_surface_unsafe(target)
                .expect("failed to create Win32 wgpu surface")
        };
        let surface: wgpu::Surface<'static> = unsafe { std::mem::transmute(surface) };

        Self {
            hwnd: child_hwnd,
            width,
            height,
            x,
            y,
            wgpu_surface: surface,
        }
    }
}

impl NativeSurfaceContext for WindowsContext {
    /// Hide the child window.
    ///
    /// # Safety
    /// Must be called on the main thread.
    fn hide_window(&self) {
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_HIDE);
        }
    }

    fn current_window_size_and_position(&self) -> (u32, u32, u32, u32) {
        (self.width, self.height, self.x, self.y)
    }

    /// Reposition and resize the child window.
    ///
    /// Win32 and CSS both use a top-left origin, so no y-axis conversion is needed.
    ///
    /// # Safety
    /// Must be called on the main thread.
    fn move_window_size_and_position(&self, x: u32, y: u32, width: u32, height: u32, _window_height: u32) {
        unsafe {
            let _ = SetWindowPos(
                self.hwnd,
                Some(HWND_TOP),
                x as i32,
                y as i32,
                width as i32,
                height as i32,
                SWP_NOACTIVATE | SWP_NOZORDER,
            );
            let _ = ShowWindow(self.hwnd, SW_SHOW);
        }
    }
}

impl WgpuSurfaceContext for WindowsContext {
    fn get_wgpu_surface(&self) -> &wgpu::Surface<'static> {
        &self.wgpu_surface
    }
}

impl SurfaceContext for WindowsContext {}

impl CursorContext for WindowsContext {
    /// No-op on Windows — cursor changes are handled via CSS `cursor` property in WebView2.
    fn push_cursor(&self, _cursor: &str) {}
    fn pop_cursor(&self) {}
}
