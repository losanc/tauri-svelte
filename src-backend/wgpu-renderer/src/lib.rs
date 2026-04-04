use native_tauri_surface::{GpuContext, SurfaceSource};
use std::borrow::Cow;
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
    pub async fn new(source: impl SurfaceSource) -> Self {
        let ctx = GpuContext::init_wgpu(source).await;
        let shader = ctx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(SHADER)),
            });
        let pipeline = ctx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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

    pub fn hide(&self) {
        self.ctx.hide();
    }

    pub fn update_frame(&self, x: f64, y: f64, width: f64, height: f64, window_height: f64) {
        self.ctx.update_frame(x, y, width, height, window_height);
    }

    pub fn resize(&self, width: u32, height: u32) {
        self.ctx.resize(width, height);
    }

    pub fn render(&self) {
        match self.ctx.surface().get_current_texture() {
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
            wgpu::CurrentSurfaceTexture::Suboptimal(st) => {
                st.present();
                native_tauri_surface::my_print!("wgpu: surface suboptimal, resize pending");
            }
            wgpu::CurrentSurfaceTexture::Timeout => {
                native_tauri_surface::my_print!(
                    "wgpu: get_current_texture timed out, skipping frame"
                );
            }
            wgpu::CurrentSurfaceTexture::Occluded => {}
            wgpu::CurrentSurfaceTexture::Outdated => {
                native_tauri_surface::my_print!("wgpu: surface outdated, reconfigure needed");
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                native_tauri_surface::my_print!("wgpu: surface lost, recreate needed");
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                native_tauri_surface::my_print!("wgpu: surface validation error");
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::Renderer;
    use native_tauri_surface::GpuSurface;
    use std::rc::Rc;
    use wasm_bindgen::prelude::*;
    use wgpu::web_sys::HtmlCanvasElement;

    #[wasm_bindgen]
    pub struct WasmRenderer {
        inner: Rc<Renderer>,
    }

    #[wasm_bindgen]
    impl WasmRenderer {
        pub async fn create(canvas: HtmlCanvasElement) -> WasmRenderer {
            WasmRenderer {
                inner: Rc::new(Renderer::new(canvas).await),
            }
        }

        pub fn set_rect(&self, x: f64, y: f64, width: f64, height: f64) {
            GpuSurface::set_rect(self, x, y, width, height);
        }

        pub fn render(&self) {
            GpuSurface::render(self);
        }
    }

    impl GpuSurface for WasmRenderer {
        fn set_rect(&self, _x: f64, _y: f64, width: f64, height: f64) {
            let dpr = web_sys::window()
                .map(|w| w.device_pixel_ratio())
                .unwrap_or(1.0);
            let w = (width * dpr) as u32;
            let h = (height * dpr) as u32;
            self.inner.update_frame(0.0, 0.0, w as f64, h as f64, 0.0);
            self.inner.resize(w, h);
        }

        fn render(&self) {
            self.inner.render();
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm::WasmRenderer;
