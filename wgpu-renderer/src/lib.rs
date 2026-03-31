use std::borrow::Cow;
use native_tauri_surface::{GpuContext, surface_helper::WgpuCompatibleSurface};
use wgpu::RenderPipeline;

const SHADER: &str = r#"
@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let x = f32(i32(i) - 1);
    let y = f32(i32(i & 1u) * 2 - 1);
    return vec4<f32>(x, y, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
"#;

pub struct Renderer {
    ctx: GpuContext,
    pipeline: RenderPipeline,
}

impl Renderer {
    pub async fn new(surface: impl WgpuCompatibleSurface) -> Self {
        let ctx = GpuContext::init_wgpu(surface).await;
        let shader = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(SHADER)),
        });
        let pipeline = ctx.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ctx.format.into())],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        Self { ctx, pipeline }
    }

    pub fn resize(&self, width: u32, height: u32) {
        self.ctx.resize(width, height);
    }

    pub fn render(&self) {
        match self.ctx.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(st) => {
                let view = st.texture.create_view(&Default::default());
                let mut enc = self.ctx.device.create_command_encoder(&Default::default());
                {
                    let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: wgpu::StoreOp::Store,
                            },
                            depth_slice: None,
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });
                    pass.set_pipeline(&self.pipeline);
                    pass.draw(0..3, 0..1);
                }
                self.ctx.queue.submit(Some(enc.finish()));
                st.present();
            }
            _ => {}
        }
    }
}

/// Shared surface interface, mirroring the TypeScript `GpuSurface`.
/// - Native (Tauri): implemented by the `set_surface_rect` / `render_surface` commands.
/// - WASM: implemented by `WasmRenderer`.
pub trait GpuSurface {
    /// CSS-pixel rect. x/y are screen position (ignored by WASM); width/height drive resize.
    fn set_rect(&self, x: f64, y: f64, width: f64, height: f64);
    fn render(&self);
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::spawn_local;
    use wgpu::web_sys::HtmlCanvasElement;
    use std::rc::Rc;
    use super::{GpuSurface, Renderer};

    #[wasm_bindgen]
    pub struct WasmRenderer {
        inner: Rc<Renderer>,
        canvas: HtmlCanvasElement,
    }

    #[wasm_bindgen]
    impl WasmRenderer {
        pub async fn create(canvas: HtmlCanvasElement) -> WasmRenderer {
            let renderer = Renderer::new(canvas.clone()).await;
            WasmRenderer { inner: Rc::new(renderer), canvas }
        }

        pub fn set_rect(&self, x: f64, y: f64, width: f64, height: f64) {
            GpuSurface::set_rect(self, x, y, width, height);
        }

        pub fn render(&self) {
            let r = Rc::clone(&self.inner);
            spawn_local(async move { r.render() });
        }
    }

    impl GpuSurface for WasmRenderer {
        fn set_rect(&self, _x: f64, _y: f64, width: f64, height: f64) {
            let dpr = web_sys::window().map(|w| w.device_pixel_ratio()).unwrap_or(1.0);
            let w = (width * dpr) as u32;
            let h = (height * dpr) as u32;
            if self.canvas.width() != w || self.canvas.height() != h {
                self.canvas.set_width(w);
                self.canvas.set_height(h);
                self.inner.resize(w, h);
            }
        }

        fn render(&self) {
            self.inner.render();
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm::WasmRenderer;
