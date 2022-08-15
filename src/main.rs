mod camera;
mod fileobserver;
mod raymarch_pipeline;
mod utils;

use glam::vec3;
use std::time::{Duration, Instant};

use camera::{Camera, CameraController};
use raymarch_pipeline::RayMarchPipeline;
use utils::{load_spirv_shader, ComputeUniforms};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu_sandbox::prelude::*;

const TEXTURE_WIDTH: u32 = 1280;
const TEXTURE_HEIGHT: u32 = 720;
const WORKGROUP_LOCAL_SIZE: (u32, u32) = (16, 16);
const WORKGROUP_SIZE: (u32, u32) = (
    TEXTURE_WIDTH / WORKGROUP_LOCAL_SIZE.0,
    TEXTURE_HEIGHT / WORKGROUP_LOCAL_SIZE.1,
);

#[derive(Debug)]
pub struct MainApp<'a> {
    render_pipeline: wgpu::RenderPipeline,
    render_bind_group: wgpu::BindGroup,
    indices_buffer: wgpu::Buffer,
    vertices_buffer: wgpu::Buffer,
    raymarch_pipeline: RayMarchPipeline<'a>,
    camera_controller: CameraController,
    compute_uniforms: ComputeUniforms,
    clock: Instant,
    run_shader: bool,
    enable_hot_reload: bool,
}

impl<'a> AppInstance for MainApp<'a> {
    fn create(gpu: &Gpu) -> Self {
        // loading vertices and indices buffers for the canvas
        let vertices_buffer = gpu.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("quad_vertex_buffer"),
            usage: wgpu::BufferUsages::VERTEX,
            contents: bytemuck::cast_slice(QUAD),
        });

        let indices_buffer = gpu.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("quad_index_buffer"),
            usage: wgpu::BufferUsages::INDEX,
            contents: bytemuck::cast_slice(QUAD_INDICES),
        });

        // loading shaders
        let vs_mod =
            load_spirv_shader("assets/compiled_shaders/quad.vert.spv", &gpu.device).unwrap();
        let fs_mod =
            load_spirv_shader("assets/compiled_shaders/quad.frag.spv", &gpu.device).unwrap();

        // create texture
        let render_texture = TextureBuilder::new()
            .with_format(wgpu::TextureFormat::Rgba8Unorm)
            .with_usages(wgpu::TextureUsages::STORAGE_BINDING)
            .with_data(&[0; (TEXTURE_WIDTH * TEXTURE_HEIGHT * 4) as usize])
            .build((TEXTURE_WIDTH, TEXTURE_HEIGHT), &gpu.device, &gpu.queue);

        // initialize render pipeline
        let render_pipeline_bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("render_pipeline_bind_group_layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });

        let render_pipeline_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("render_pipeline_bind_group"),
            layout: &render_pipeline_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&render_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&render_texture.sampler),
                },
            ],
        });

        // building render pipeline
        let pipeline_layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("main_render_pipeline_layout"),
                bind_group_layouts: &[&render_pipeline_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("main_render_pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &vs_mod,
                    entry_point: "main",
                    buffers: &[Vertex2D::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &fs_mod,
                    entry_point: "main",
                    targets: &[wgpu::ColorTargetState {
                        format: gpu.get_surface_texture_format(),
                        blend: None,
                        write_mask: wgpu::ColorWrites::all(),
                    }],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    ..Default::default()
                },
                depth_stencil: None,
                multiview: None,
                multisample: wgpu::MultisampleState::default(),
            });

        // init raymarching context
        let raymarch_pipeline = RayMarchPipeline::new(&gpu.device, &render_texture);
        let camera_controller = CameraController::new(
            Camera::new(vec3(5.0, 5.0, 5.0), vec3(0.0, 0.0, 0.0), 1.5),
            (TEXTURE_WIDTH as f32, TEXTURE_HEIGHT as f32),
        );
        let compute_uniforms = ComputeUniforms::new(camera_controller.camera, 0.0);
        raymarch_pipeline.upload_uniforms(&gpu, &compute_uniforms);

        Self {
            render_pipeline,
            vertices_buffer,
            indices_buffer,
            render_bind_group: render_pipeline_bind_group,
            raymarch_pipeline,
            camera_controller,
            compute_uniforms,
            clock: Instant::now(),
            run_shader: true,
            enable_hot_reload: true,
        }
    }

    fn render(&self, gpu: &Gpu, frame: &wgpu::SurfaceTexture) {
        let mut render_encoder =
            gpu.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("render_encoder"),
                });
        let frame_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // render pass
        {
            let mut rpass = render_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main_render_pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &frame_view,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                    resolve_target: None,
                }],
                depth_stencil_attachment: None,
            });

            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_vertex_buffer(0, self.vertices_buffer.slice(..));
            rpass.set_index_buffer(self.indices_buffer.slice(..), wgpu::IndexFormat::Uint32);
            rpass.set_bind_group(0, &self.render_bind_group, &[]);
            rpass.draw_indexed(0..QUAD_INDICES.len() as u32, 0, 0..1);
        }

        gpu.queue.submit(std::iter::once(render_encoder.finish()));
    }

    fn events(&mut self, event: &winit::event::WindowEvent) {
        self.camera_controller.handle_events(event);
    }

    fn update(&mut self, gpu: &Gpu, dt: Duration) {
        let time = self.clock.elapsed().as_secs_f32();
        self.camera_controller.update(dt);

        // update uniforms
        self.compute_uniforms.update_time(time);
        self.compute_uniforms
            .update_camera(self.camera_controller.camera);
        self.raymarch_pipeline
            .upload_uniforms(&gpu, &self.compute_uniforms);

        if self.enable_hot_reload {
            self.raymarch_pipeline.update_shader(&gpu);
        }

        if self.run_shader {
            self.raymarch_pipeline.execute(&gpu, WORKGROUP_SIZE);
        }
    }

    fn on_imgui(&mut self, ui: &imgui::Ui, _gpu: &Gpu, dt: Duration) {
        let dt = dt.as_secs_f32();

        imgui::Window::new("Control").build(&ui, || {
            ui.text(format!(
                "frame time : {0:.1}ms ({1:.0} fps)",
                dt * 1000.0,
                1.0 / dt
            ));
            ui.separator();
            ui.text("Hello");
            ui.checkbox("run", &mut self.run_shader);
            ui.checkbox("hot reload", &mut self.enable_hot_reload);
        });
    }
}

fn main() {
    AppBuilder::new()
        .with_name("Ray marching")
        .with_dimension(TEXTURE_WIDTH, TEXTURE_HEIGHT)
        .with_resizable(true)
        .build()
        .run::<MainApp>()
}
