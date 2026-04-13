use native_tauri_surface::{GpuContext, SurfaceSource};
use std::{borrow::Cow, collections::HashMap};
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
    pipeline: HashMap<u64, RenderPipeline>,
    ctx: GpuContext,
}

impl Renderer {
    pub async fn new() -> Self {
        let ctx = GpuContext::init().await;

        Self {
            ctx,
            pipeline: HashMap::new(),
        }
    }

    /// use native resolution. so width height are always integer
    pub async fn add_surface(
        &mut self,
        source: impl SurfaceSource,
        width: u32,
        height: u32,
        x: u32,
        y: u32,
    ) -> u64 {
        let hash = self.ctx.add_surface(source, width, height, x, y).await;

        let shader = self
            .ctx
            .device()
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(SHADER)),
            });
        let pipeline = self
            .ctx
            .device()
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
                    targets: &[Some(
                        self.ctx
                            .surfaces()
                            .get(&hash)
                            .expect("impossible")
                            .current_format()
                            .into(),
                    )],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            });

        self.pipeline.insert(hash, pipeline);
        self.render_surface(hash);
        hash
    }

    pub async fn remove_surface(&mut self, hash: u64) {
        self.ctx.remove_surface(hash);
        self.pipeline.remove(&hash);
    }

    pub fn hide_surface(&self, hash: u64) {
        if let Some(surface) = self.ctx.surfaces().get(&hash) {
            surface.hide_window();
        } else {
            println!("?? {hash}");
            native_tauri_surface::my_print!("can't find it");
        }
    }

    pub fn show_surface(&self, hash: u64) {
        if let Some(surface) = self.ctx.surfaces().get(&hash) {
            surface.show_window();
        } else {
            println!("!! {hash}");

            native_tauri_surface::my_print!("can't find it");
        }
    }

    pub fn destroy_surface(&mut self, hash: u64) {
        self.ctx.remove_surface(hash);
    }
    pub fn set_render_resolution(&self, hash: u64, width: u32, height: u32) {
        if let Some(surface) = self.ctx.surfaces().get(&hash) {
            surface.set_render_resolution(self.ctx.device(), self.ctx.adapter(), width, height);
        } else {
            native_tauri_surface::my_print!("can't find it");
        }
    }

    pub fn set_surface_position(&mut self, hash: u64, width: u32, height: u32, x: u32, y: u32) {
        self.ctx.move_surface(hash, width, height, x, y);
        self.render_surface(hash);
    }
    pub fn render_surface(&self, hash: u64) {
        if let Some(surface) = self.ctx.surfaces().get(&hash)
            && let Some(pipeline) = self.pipeline.get(&hash)
        {
            match surface.get_wgpu_surface().get_current_texture() {
                wgpu::CurrentSurfaceTexture::Success(st) => {
                    let view = st.texture.create_view(&Default::default());
                    let mut enc = self
                        .ctx
                        .device()
                        .create_command_encoder(&Default::default());
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
                        pass.set_pipeline(&pipeline);
                        pass.draw(0..3, 0..1);
                    }
                    self.ctx.queue().submit(Some(enc.finish()));
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
}

// #[cfg(target_arch = "wasm32")]
// mod wasm {
//     use super::Renderer;
//     use native_tauri_surface::GpuSurface;
//     use std::rc::Rc;
//     use wasm_bindgen::prelude::*;
//     use wgpu::web_sys::HtmlCanvasElement;

//     #[wasm_bindgen]
//     pub struct WasmRenderer {
//         inner: Rc<Renderer>,
//     }

//     #[wasm_bindgen]
//     impl WasmRenderer {
//         pub async fn create(canvas: HtmlCanvasElement) -> WasmRenderer {
//             WasmRenderer {
//                 inner: Rc::new(
//                     Renderer::new(canvas.clone(), canvas.width(), canvas.height(), 0, 0).await,
//                 ),
//             }
//         }

//         pub fn set_rect(&self, x: f64, y: f64, width: f64, height: f64) {
//             GpuSurface::set_rect(self, x, y, width, height);
//         }

//         pub fn render(&self) {
//             GpuSurface::render(self);
//         }
//     }

//     impl GpuSurface for WasmRenderer {
//         fn set_rect(&self, _x: f64, _y: f64, width: f64, height: f64) {
//             let dpr = web_sys::window()
//                 .map(|w| w.device_pixel_ratio())
//                 .unwrap_or(1.0);
//             let w = (width * dpr) as u32;
//             let h = (height * dpr) as u32;
//             self.inner.update_frame(0.0, 0.0, w as f64, h as f64, 0.0);
//             self.inner.resize(w, h);
//         }

//         fn render(&self) {
//             self.inner.render();
//         }
//     }
// }

// #[cfg(target_arch = "wasm32")]
// pub use wasm::WasmRenderer;
