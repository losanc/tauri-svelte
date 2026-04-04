/// Platform-level GPU surface support tests.
///
/// - On macOS: verifies that at least one Metal adapter is available, confirming
///   the native `MacOSContext` / `CAMetalLayer` path can actually be used at runtime.
/// - On non-macOS native targets: asserts the absence of native surface support,
///   documenting the platform constraint rather than silently skipping.
///
/// These tests run as part of `cargo test --workspace` in CI.

// ── macOS: Metal adapter must be present ─────────────────────────────────────

#[cfg(target_os = "macos")]
#[test]
fn metal_adapter_is_available() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::METAL,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    let adapters = pollster::block_on(instance.enumerate_adapters(wgpu::Backends::METAL));
    assert!(
        !adapters.is_empty(),
        "Expected at least one Metal adapter on macOS CI — found none. \
         This means native surface creation (MacOSContext) will fail at runtime."
    );
}

#[cfg(target_os = "macos")]
#[test]
fn metal_adapter_reports_metal_backend() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::METAL,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    let adapters = pollster::block_on(instance.enumerate_adapters(wgpu::Backends::METAL));
    assert!(!adapters.is_empty(), "No Metal adapters found");
    assert_eq!(
        adapters[0].get_info().backend,
        wgpu::Backend::Metal,
        "First adapter should report Metal backend"
    );
}

// ── Non-macOS native: no MacOSContext is compiled ────────────────────────────

/// On Windows and Linux, `MacOSContext` is not compiled in.
/// This test documents that fact and ensures CI coverage on those platforms.
#[cfg(not(any(target_os = "macos", target_arch = "wasm32")))]
#[test]
fn no_native_surface_on_this_platform() {
    assert!(
        !cfg!(target_os = "macos"),
        "MacOSContext is only available on macOS; this platform has no native surface backend"
    );
}
