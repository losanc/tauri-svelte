// Platform-specific imports needed by SurfaceOwner at the top level.
#[cfg(not(target_arch = "wasm32"))]
use {objc2::rc::Retained, objc2_app_kit::NSView, objc2_quartz_core::CAMetalLayer};

/// Owns the native resources that the wgpu surface's raw pointer points into.
///
/// Stored inside [`GpuContext`](crate::GpuContext) alongside the `Surface`, with field
/// declaration order guaranteeing the surface drops before this owner. Variants are
/// platform-exclusive — only one is compiled per target.
///
/// # Safety
///
/// Fields are only accessed on their respective thread:
/// - **Native:** main thread only, enforced via `run_on_main_thread`.
/// - **WASM:** single-threaded environment.
pub enum SurfaceOwner {
    #[cfg(target_arch = "wasm32")]
    Wasm {
        canvas: wgpu::web_sys::HtmlCanvasElement,
    },
    #[cfg(not(target_arch = "wasm32"))]
    Native {
        /// Retains the NSView that hosts the Metal layer.
        view: Retained<NSView>,
        /// Retains the CAMetalLayer whose raw pointer wgpu holds.
        /// Dropping this while the surface exists would be UB.
        layer: Retained<CAMetalLayer>,
    },
}

// Safety: all fields are only accessed on the appropriate thread.
// Native: main thread only, enforced via run_on_main_thread.
// Wasm: single-threaded environment.
unsafe impl Send for SurfaceOwner {}
unsafe impl Sync for SurfaceOwner {}

impl SurfaceOwner {
    /// Returns a [`SurfaceResizer`] — a lightweight clone of the native handles
    /// for dispatching AppKit frame and visibility operations to the main thread.
    ///
    /// Only available on native (non-WASM) targets.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn resizer(&self) -> native::SurfaceResizer {
        match self {
            SurfaceOwner::Native { view, layer } => native::SurfaceResizer {
                ns_view: view.clone(),
                metal_layer: layer.clone(),
            },
        }
    }

    /// Returns the `HtmlCanvasElement` that backs this surface.
    ///
    /// Only available on WASM targets.
    #[cfg(target_arch = "wasm32")]
    pub fn canvas(&self) -> &wgpu::web_sys::HtmlCanvasElement {
        match self {
            SurfaceOwner::Wasm { canvas } => canvas,
        }
    }
}

/// Creates a wgpu surface from a platform-specific source and returns
/// the [`SurfaceOwner`] that keeps the underlying native resources alive alongside it.
pub trait SurfaceSource {
    /// Consume this source, create a wgpu surface via `instance`, and return:
    /// - the [`SurfaceOwner`] that must outlive the surface,
    /// - the surface itself,
    /// - and the initial `(width, height)` in physical pixels.
    fn create(self, instance: &wgpu::Instance) -> (SurfaceOwner, wgpu::Surface<'_>, u32, u32);
}

#[cfg(target_arch = "wasm32")]
pub mod web {
    use crate::surface_helper::{SurfaceOwner, SurfaceSource};
    use wgpu::SurfaceTarget;
    use wgpu::web_sys::HtmlCanvasElement;

    impl SurfaceSource for HtmlCanvasElement {
        fn create(self, instance: &wgpu::Instance) -> (SurfaceOwner, wgpu::Surface<'_>, u32, u32) {
            let (w, h) = (self.width(), self.height());
            // Clone the JS handle for wgpu; the Wasm variant retains the original.
            // Both refer to the same underlying DOM canvas node.
            let surface = instance
                .create_surface(SurfaceTarget::Canvas(self.clone()))
                .expect("create surface failed");
            (SurfaceOwner::Wasm { canvas: self }, surface, w, h)
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub mod native {
    use crate::surface_helper::{SurfaceOwner, SurfaceSource};
    use objc2::rc::Retained;
    use objc2_app_kit::NSView;
    use objc2_quartz_core::CAMetalLayer;
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};

    /// A lightweight clone of the native Metal layer handles, used to dispatch
    /// AppKit frame and visibility operations to the main thread via
    /// `app.run_on_main_thread(...)`.
    ///
    /// Obtained from [`SurfaceOwner::resizer`] or [`GpuContext::resizer`](crate::GpuContext::resizer).
    /// Cheap to clone — both fields are Objective-C `Retained` (reference-counted) pointers.
    pub struct SurfaceResizer {
        pub ns_view: Retained<NSView>,
        pub metal_layer: Retained<CAMetalLayer>,
    }

    // Safety: only accessed on the main thread via run_on_main_thread.
    unsafe impl Send for SurfaceResizer {}
    unsafe impl Sync for SurfaceResizer {}

    impl SurfaceResizer {
        /// Hide the Metal layer by setting the NSView hidden flag.
        ///
        /// Call this when the panel has zero size or is not visible.
        ///
        /// # Safety
        ///
        /// Must be called on the main thread.
        pub unsafe fn hide(&self) {
            self.ns_view.setHidden(true);
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
        pub unsafe fn update_frame(
            &self,
            x: f64,
            y: f64,
            width: f64,
            height: f64,
            window_height: f64,
        ) {
            use objc2_foundation::{NSPoint, NSRect, NSSize};
            let mac_y = window_height - y - height;
            self.ns_view.setFrame(NSRect::new(
                NSPoint::new(x, mac_y),
                NSSize::new(width, height),
            ));
            self.ns_view.setHidden(false);
        }
    }

    /// Push a resize cursor onto the macOS NSCursor stack.
    ///
    /// Pushed cursors take precedence over NSWindow cursor rects (including
    /// WKWebView's), ensuring the correct resize icon appears over dockview sashes
    /// regardless of CSS cursor values. Pair every push with [`pop_cursor`].
    ///
    /// # Safety
    ///
    /// Must be called on the main thread.
    pub fn push_resize_cursor(horizontal: bool) {
        unsafe {
            use objc2::runtime::AnyObject;
            use objc2::{class, msg_send};
            let cursor: *const AnyObject = if horizontal {
                msg_send![class!(NSCursor), resizeLeftRightCursor]
            } else {
                msg_send![class!(NSCursor), resizeUpDownCursor]
            };
            let _: () = msg_send![cursor, push];
        }
    }

    /// Pop the top cursor from the macOS NSCursor stack.
    ///
    /// Must be paired with a prior call to [`push_resize_cursor`].
    ///
    /// # Safety
    ///
    /// Must be called on the main thread.
    pub fn pop_cursor() {
        unsafe {
            use objc2::{class, msg_send};
            let _: () = msg_send![class!(NSCursor), pop];
        }
    }

    /// Constructs a Metal-backed NSView and wires it into the Tauri window hierarchy.
    ///
    /// Creates a new `NSView` at the specified position, attaches a `CAMetalLayer`,
    /// and adds it as a subview of the window's content view. The resulting
    /// `SurfaceHelper` can be passed to [`GpuContext::init_wgpu`](crate::GpuContext::init_wgpu)
    /// via the [`SurfaceSource`] impl to obtain a fully configured wgpu surface.
    ///
    /// Currently only implemented for macOS (`AppKit` window handles).
    pub struct SurfaceHelper {
        surface: wgpu::SurfaceTargetUnsafe,
        metal_view: Retained<NSView>,
        metal_layer: Retained<CAMetalLayer>,
        width: u32,
        height: u32,
    }

    impl SurfaceHelper {
        /// Create a new Metal NSView subview inside the given Tauri window.
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
            let handle = window.window_handle().unwrap();
            match handle.as_raw() {
                #[cfg(target_os = "macos")]
                RawWindowHandle::AppKit(app_kit_window_handle) => {
                    use objc2::runtime::AnyObject;
                    use objc2::{MainThreadMarker, MainThreadOnly, rc::Retained};
                    use objc2_app_kit::NSView;
                    use objc2_foundation::{NSPoint, NSRect, NSSize};

                    let ns_view_ptr = app_kit_window_handle.ns_view.as_ptr() as *mut AnyObject;
                    let ns_view = unsafe { &(*ns_view_ptr) };
                    let view = ns_view.downcast_ref::<NSView>().expect("invalid NSView");
                    let window = view.window().expect("failed to get window");
                    let context_view = window.contentView().expect("failed to create content view");

                    let metal_rect = NSRect::new(
                        NSPoint::new(position_x as _, position_y as _),
                        NSSize::new(width as _, height as _),
                    );
                    let mtm = MainThreadMarker::new().expect("must be on the main thread");
                    let metal_view = NSView::initWithFrame(NSView::alloc(mtm), metal_rect);
                    metal_view.setWantsLayer(true);
                    let metal_layer = CAMetalLayer::new();
                    metal_view.setLayer(Some(&metal_layer));
                    context_view.addSubview(&metal_view);

                    let layer_ptr = Retained::as_ptr(&metal_layer) as *mut std::ffi::c_void;
                    let target = wgpu::SurfaceTargetUnsafe::CoreAnimationLayer(layer_ptr);

                    Self {
                        surface: target,
                        metal_view,
                        metal_layer,
                        width,
                        height,
                    }
                }
                _ => unimplemented!(),
            }
        }
    }

    impl SurfaceSource for SurfaceHelper {
        fn create(self, instance: &wgpu::Instance) -> (SurfaceOwner, wgpu::Surface<'_>, u32, u32) {
            let owner = SurfaceOwner::Native {
                view: self.metal_view,
                layer: self.metal_layer,
            };
            let surface = unsafe {
                instance
                    .create_surface_unsafe(self.surface)
                    .expect("failed to create surface")
            };
            (owner, surface, self.width, self.height)
        }
    }
}
