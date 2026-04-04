use crate::platform::surface_context::{CursorContext, SurfaceContext, SurfaceSource};

use objc2::rc::Retained;
use objc2_app_kit::NSView;
use objc2_quartz_core::CAMetalLayer;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};

/// macOS GPU surface backed by an `NSView` and a `CAMetalLayer`.
///
/// Holds `Retained` references to both the view and the Metal layer so they stay
/// alive as long as the context does.  `Send + Sync` is asserted manually because
/// `Retained<NSView>` is not automatically `Send`; callers must ensure all AppKit
/// operations happen on the main thread.
#[cfg(target_os = "macos")]
pub struct MacOSContext {
    view: Retained<NSView>,
    layer: Retained<CAMetalLayer>,
}

unsafe impl Send for MacOSContext {}
unsafe impl Sync for MacOSContext {}

impl MacOSContext {
    /// Create a Metal-backed `NSView` subview inside the given Tauri window.
    ///
    /// Allocates a new `NSView` at the specified position, attaches a `CAMetalLayer`,
    /// and adds it as a subview of the window's content view. Pass the returned
    /// `MacOSContext` to [`GpuContext::init_wgpu`](crate::GpuContext::init_wgpu)
    /// via the [`SurfaceSource`] impl to obtain a fully configured wgpu surface.
    ///
    /// # Parameters
    ///
    /// - `window` — any type that provides a raw window handle (e.g. `tauri::WebviewWindow`).
    /// - `width`, `height` — initial surface size in physical pixels.
    /// - `position_x`, `position_y` — initial position within the window in physical pixels.
    ///
    /// # Panics
    ///
    /// Panics if the window handle is not an AppKit handle, or if the NSView / window
    /// hierarchy cannot be retrieved.
    pub fn new(
        window: &impl HasWindowHandle,
        width: u32,
        height: u32,
        position_x: u32,
        position_y: u32,
    ) -> Self {
        use objc2::runtime::AnyObject;
        use objc2::{MainThreadMarker, MainThreadOnly};
        use objc2_foundation::{NSPoint, NSRect, NSSize};

        let handle = window.window_handle().unwrap();
        let RawWindowHandle::AppKit(h) = handle.as_raw() else {
            panic!("expected AppKit handle");
        };

        let ns_view_ptr = h.ns_view.as_ptr() as *mut AnyObject;
        let ns_view = unsafe { &(*ns_view_ptr) };
        let view = ns_view.downcast_ref::<NSView>().expect("invalid NSView");
        let window = view.window().expect("failed to get window");
        let content_view = window.contentView().expect("no content view");

        let metal_rect = NSRect::new(
            NSPoint::new(position_x as f64, position_y as f64),
            NSSize::new(width as f64, height as f64),
        );
        let mtm = MainThreadMarker::new().expect("must be on main thread");
        let metal_view = NSView::initWithFrame(NSView::alloc(mtm), metal_rect);
        metal_view.setWantsLayer(true);
        let metal_layer = CAMetalLayer::new();
        metal_view.setLayer(Some(&metal_layer));
        content_view.addSubview(&metal_view);

        Self {
            view: metal_view,
            layer: metal_layer,
        }
    }

    /// Exposes the raw CAMetalLayer pointer for wgpu surface creation.
    pub fn surface_target(&self) -> wgpu::SurfaceTargetUnsafe {
        let layer_ptr = Retained::as_ptr(&self.layer) as *mut std::ffi::c_void;
        wgpu::SurfaceTargetUnsafe::CoreAnimationLayer(layer_ptr)
    }
}

impl SurfaceSource for MacOSContext {
    type Context = MacOSContext;

    /// Create a wgpu surface from the `CAMetalLayer` and return the context alongside it.
    ///
    /// # Safety
    ///
    /// The `Surface<'static>` lifetime is obtained via `transmute`. The [`GpuContext`](crate::GpuContext)
    /// field ordering ensures the surface is dropped before `MacOSContext`, keeping the
    /// raw pointer valid for the surface's entire lifetime.
    fn create(self, instance: &wgpu::Instance) -> (MacOSContext, wgpu::Surface<'static>) {
        let target = self.surface_target();
        let surface = unsafe {
            instance
                .create_surface_unsafe(target)
                .expect("failed to create surface")
        };

        let surface_ctx: wgpu::Surface<'static> = unsafe { std::mem::transmute(surface) };
        (self, surface_ctx)
    }
}

impl SurfaceContext for MacOSContext {
    /// Returns the current `NSView` frame size in physical pixels.
    fn initial_size(&self) -> (u32, u32) {
        let frame = self.view.frame();
        (frame.size.width as u32, frame.size.height as u32)
    }

    /// Hide the Metal layer by setting the NSView hidden flag.
    ///
    /// Call this when the panel has zero size or is not visible.
    ///
    /// # Safety
    ///
    /// Must be called on the main thread.
    fn hide(&self) {
        self.view.setHidden(true);
    }

    /// Reposition and resize the Metal NSView within its parent window.
    ///
    /// Converts from CSS pixel coordinates (top-left origin, `y` increasing downward)
    /// to AppKit coordinates (bottom-left origin, `y` increasing upward) using
    /// `window_height`.
    ///
    /// # Parameters
    ///
    /// - `x`, `y` — panel position in logical (CSS) pixels, top-left origin.
    /// - `width`, `height` — panel size in logical pixels.
    /// - `window_height` — logical height of the window's inner content area,
    ///   used to flip the y-axis for AppKit.
    ///
    /// # Safety
    ///
    /// Must be called on the main thread.
    fn update_frame(&self, x: f64, y: f64, width: f64, height: f64, window_height: f64) {
        use objc2_foundation::{NSPoint, NSRect, NSSize};
        let mac_y = window_height - y - height;
        self.view.setFrame(NSRect::new(
            NSPoint::new(x, mac_y),
            NSSize::new(width, height),
        ));
        self.view.setHidden(false);
    }
}

impl CursorContext for MacOSContext {
    fn push_cursor(&self, cursor: &str) {
        push_cursor(cursor);
    }

    fn pop_cursor(&self) {
        pop_cursor();
    }
}

/// Push a named cursor onto the macOS NSCursor stack.
///
/// Accepts `"ew-resize"` and `"ns-resize"`; ignores unknown values.
/// Must be called on the main thread.
pub fn push_cursor(cursor: &str) {
    use objc2::runtime::AnyObject;
    use objc2::{class, msg_send};
    let cursor_obj: *const AnyObject = match cursor {
        "ew-resize" => unsafe { msg_send![class!(NSCursor), resizeLeftRightCursor] },
        "ns-resize" => unsafe { msg_send![class!(NSCursor), resizeUpDownCursor] },
        _ => return,
    };
    unsafe {
        let _: () = msg_send![cursor_obj, push];
    }
}

/// Pop the top cursor from the macOS NSCursor stack.
///
/// Must be paired with a prior call to [`push_cursor`].
/// Must be called on the main thread.
pub fn pop_cursor() {
    use objc2::{class, msg_send};
    unsafe {
        let _: () = msg_send![class!(NSCursor), pop];
    }
}
