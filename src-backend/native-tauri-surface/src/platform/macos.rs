use crate::platform::surface_context::CursorContext;
use crate::{NativeSurfaceContext, SurfaceContext, SurfaceHash, WgpuSurfaceContext};

use objc2::rc::Retained;
use objc2_app_kit::NSView;
use objc2_quartz_core::CAMetalLayer;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::hash::{DefaultHasher, Hash, Hasher};
use wgpu::Instance;

/// macOS GPU surface backed by an `NSView` subview with a `CAMetalLayer`.
///
/// A new `NSView` is created inside the Tauri window's content view and given a
/// `CAMetalLayer` as its backing layer. wgpu renders to that layer via a Metal surface.
///
/// `Send + Sync` are asserted manually because `Retained<NSView>` is not automatically
/// `Send`. All AppKit operations (`setFrame`, `setHidden`) must happen on the main thread.
#[cfg(target_os = "macos")]
pub struct MacOSContext {
    wgpu_surface: wgpu::Surface<'static>,
    #[allow(dead_code)]
    layer: Retained<CAMetalLayer>,
    view: Retained<NSView>,
}

unsafe impl Send for MacOSContext {}
unsafe impl Sync for MacOSContext {}

impl Drop for MacOSContext
{
    fn drop(&mut self) {
        self.layer.removeFromSuperlayer();
        self.view.removeFromSuperview();
    }
}

impl MacOSContext {
    /// Create a Metal-backed `NSView` subview inside the given Tauri window.
    ///
    /// Allocates a new `NSView` at the specified position, attaches a `CAMetalLayer`,
    /// adds it as a subview of the window's content view, and creates the wgpu Metal
    /// surface from the layer.

    /// Panics if the window handle is not an AppKit handle, or if the NSView / window
    /// hierarchy cannot be retrieved.
    ///
    /// Must be called on the main thread.
    pub fn new(
        instance: &Instance,
        window: &impl HasWindowHandle,
        width: u32,
        height: u32,
        x: u32,
        y: u32,
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
        // Convert CSS y coordinate to AppKit y coordinate using the content view height, so the initial position is correct.
        let content_height = content_view.frame().size.height;
        let mac_y = css_y_to_appkit(y as f64, height as f64, content_height);

        let metal_rect = NSRect::new(
            NSPoint::new(x as f64, mac_y),
            NSSize::new(width as f64, height as f64),
        );
        let mtm = MainThreadMarker::new().expect("must be on main thread");
        let metal_view = NSView::initWithFrame(NSView::alloc(mtm), metal_rect);
        metal_view.setWantsLayer(true);
        let metal_layer = CAMetalLayer::new();
        metal_view.setLayer(Some(&metal_layer));
        content_view.addSubview(&metal_view);

        // Create the wgpu Metal surface from the CAMetalLayer raw pointer.
        // `metal_layer` is held by `metal_view` which is retained below, so the
        // layer pointer remains valid for the lifetime of this struct.
        let layer_ptr = Retained::as_ptr(&metal_layer) as *mut std::ffi::c_void;
        let target = wgpu::SurfaceTargetUnsafe::CoreAnimationLayer(layer_ptr);
        let surface = unsafe {
            instance
                .create_surface_unsafe(target)
                .expect("failed to create Metal surface")
        };
        // SAFETY: `metal_layer` (and the raw pointer the surface holds) lives as long
        // as `MacOSContext` because `layer` is a field declared after `wgpu_surface`.
        let wgpu_surface: wgpu::Surface<'static> = unsafe { std::mem::transmute(surface) };

        Self {
            wgpu_surface,
            view: metal_view,
            layer: metal_layer,
        }
    }
}

impl WgpuSurfaceContext for MacOSContext {
    fn get_wgpu_surface(&self) -> &wgpu::Surface<'static> {
        &self.wgpu_surface
    }
}

impl Drop for MacOSContext {
    fn drop(&mut self) {
        unsafe { self.view.removeFromSuperview() };
    }
}

impl NativeSurfaceContext for MacOSContext {
    /// Hide the Metal NSView.
    fn hide_window(&self) {
        self.view.setHidden(true);
    }

    fn show_window(&self) {
        self.view.setHidden(false);
    }

    /// Returns the live NSView frame as `(width, height, x, y)` in physical pixels.
    ///
    /// Reads directly from the NSView, so it reflects the most recent `setFrame` call.
    /// Note: AppKit y-coordinates (bottom-left origin) are returned as-is.
    fn current_window_size_and_position(&self) -> (u32, u32, u32, u32) {
        let frame = self.view.frame();
        (
            frame.size.width as u32,
            frame.size.height as u32,
            frame.origin.x as u32,
            frame.origin.y as u32,
        )
    }

    /// Reposition and resize the Metal NSView, then make it visible.
    /// Must be called on the main thread.
    fn move_window_size_and_position(&mut self, width: u32, height: u32, x: u32, y: u32) {
        use objc2_foundation::{NSPoint, NSRect, NSSize};
        let window_height = self
            .view
            .window()
            .and_then(|w| Some(w.frame().size.height))
            .unwrap_or(0.0);
        let mac_y = css_y_to_appkit(y as f64, height as f64, window_height);
        self.view.setFrame(NSRect::new(
            NSPoint::new(x as f64, mac_y),
            NSSize::new(width as f64, height as f64),
        ));
        self.view.setHidden(false);
    }
}

impl SurfaceContext for MacOSContext {
    fn hash(&self) -> SurfaceHash {
        let ptr = Retained::as_ptr(&self.view) as usize;
        let mut hasher = DefaultHasher::new();
        ptr.hash(&mut hasher);
        let result = hasher.finish();
        println!("created window hahs: {result}");
        result.into()
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

/// Convert a CSS top-left y coordinate to an AppKit bottom-left y coordinate.
///
/// CSS uses a top-left origin (y increases downward); AppKit uses a bottom-left
/// origin (y increases upward). This function converts between the two.
fn css_y_to_appkit(y: f64, height: f64, window_height: f64) -> f64 {
    window_height - y - height
}

#[cfg(test)]
mod tests {
    use super::css_y_to_appkit;

    #[test]
    fn panel_at_top_of_window() {
        // CSS y=0 → AppKit y = window_height - panel_height
        assert_eq!(css_y_to_appkit(0.0, 100.0, 800.0), 700.0);
    }

    #[test]
    fn panel_at_bottom_of_window() {
        // CSS y = window_height - height → AppKit y = 0
        assert_eq!(css_y_to_appkit(700.0, 100.0, 800.0), 0.0);
    }

    #[test]
    fn panel_fills_full_window_height() {
        // A panel that fills the entire window height always has AppKit y = 0
        assert_eq!(css_y_to_appkit(0.0, 800.0, 800.0), 0.0);
    }

    #[test]
    fn panel_at_arbitrary_offset() {
        assert_eq!(css_y_to_appkit(200.0, 150.0, 800.0), 450.0);
    }

    #[test]
    fn appkit_y_decreases_as_css_y_increases() {
        // Moving a panel down in CSS (larger y) must move it down in AppKit (smaller y)
        let y_upper = css_y_to_appkit(100.0, 50.0, 800.0);
        let y_lower = css_y_to_appkit(200.0, 50.0, 800.0);
        assert!(y_lower < y_upper);
    }
}
