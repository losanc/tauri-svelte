use crate::platform::surface_context::{CursorContext, SurfaceContext, SurfaceSource};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows::Win32::Foundation::HWND;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, HWND_TOP, SW_HIDE, SW_SHOW, SWP_NOACTIVATE, SWP_NOZORDER, SetWindowPos,
    ShowWindow, WINDOW_STYLE, WS_CHILD, WS_VISIBLE,
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
    pub fn new(window: &impl HasWindowHandle, width: u32, height: u32, x: u32, y: u32) -> Self {
        let handle = window.window_handle().unwrap();
        let RawWindowHandle::Win32(h) = handle.as_raw() else {
            panic!("expected Win32 handle");
        };

        let parent_hwnd = HWND(h.hwnd.get() as *mut std::ffi::c_void);

        // A null class name re-uses the parent's WNDCLASS which is sufficient for
        // a child surface that only serves as a wgpu render target.
        let hwnd = unsafe {
            CreateWindowExW(
                Default::default(),                      // dwExStyle
                windows::core::w!("Static"), // lpClassName — built-in, always registered
                windows::core::w!(""),       // lpWindowName
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

        Self {
            hwnd,
            width,
            height,
        }
    }
}

impl SurfaceSource for WindowsContext {
    type Context = WindowsContext;

    /// Create a wgpu surface from the child `HWND`.
    ///
    /// # Safety
    /// The `Surface<'static>` lifetime is obtained via `transmute`. `GpuContext`'s
    /// field ordering ensures the surface drops before `WindowsContext`, keeping
    /// the `HWND` valid for the surface's entire lifetime.
    fn create(self, instance: &wgpu::Instance) -> (WindowsContext, wgpu::Surface<'static>) {
        let hinstance =
            unsafe { GetModuleHandleW(None).expect("GetModuleHandleW failed").0 as isize };
        let hwnd_isize = self.hwnd.0 as isize;

        let target = wgpu::SurfaceTargetUnsafe::Win32 {
            hinstance: Some(hinstance as *mut std::ffi::c_void),
            hwnd: hwnd_isize as *mut std::ffi::c_void,
        };

        let surface = unsafe {
            instance
                .create_surface_unsafe(target)
                .expect("failed to create Win32 wgpu surface")
        };
        let surface: wgpu::Surface<'static> = unsafe { std::mem::transmute(surface) };
        (self, surface)
    }
}

impl SurfaceContext for WindowsContext {
    fn initial_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Hide the child window.
    ///
    /// # Safety
    /// Must be called on the main thread.
    fn hide(&self) {
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_HIDE);
        }
    }

    /// Reposition and resize the child window.
    ///
    /// Win32 and CSS both use a top-left origin, so no y-axis conversion is needed.
    ///
    /// # Safety
    /// Must be called on the main thread.
    fn update_frame(&self, x: f64, y: f64, width: f64, height: f64, _window_height: f64) {
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

impl CursorContext for WindowsContext {
    /// No-op on Windows — cursor changes are handled via CSS `cursor` property in WebView2.
    fn push_cursor(&self, _cursor: &str) {}
    fn pop_cursor(&self) {}
}
