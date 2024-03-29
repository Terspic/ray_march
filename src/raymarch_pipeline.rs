use shaderc;
use std::fs;

use crate::{
    filewatcher::*,
    utils::{load_spirv_shader, ComputeUniforms},
    wgpu,
};
use wgpu_sandbox::prelude::{Gpu, Texture};

#[derive(Debug)]
pub struct RayMarchPipeline<'a> {
    pipeline_layout: wgpu::PipelineLayout,
    pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
    uniforms_buffer: wgpu::Buffer,
    shader_observer: FileWatcher<'a>,
    compiler: shaderc::Compiler,
}

impl<'a> RayMarchPipeline<'a> {
    pub fn new(device: &wgpu::Device, output_texture: &Texture) -> Self {
        let uniforms = ComputeUniforms::default();
        let uniforms_buffer = uniforms.build_buffer(device);

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("compute_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        access: wgpu::StorageTextureAccess::WriteOnly,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        min_binding_size: None,
                        has_dynamic_offset: false,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("compute_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("compute_bind_group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&output_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(
                        uniforms_buffer.as_entire_buffer_binding(),
                    ),
                },
            ],
        });

        let shader_mod =
            load_spirv_shader("./assets/compiled_shaders/main.glsl.spv", device).unwrap();
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("main_compute_pipeline"),
            module: &shader_mod,
            entry_point: "main",
            layout: Some(&pipeline_layout),
        });

        let shader_observer = FileWatcher::new(&[
            "./assets/shaders/main.glsl",
            "./assets/shaders/sdf.glsl",
            "./assets/shaders/utils.glsl",
        ])
        .unwrap();

        let compiler = shaderc::Compiler::new().unwrap();

        Self {
            pipeline_layout,
            shader_observer,
            pipeline,
            bind_group,
            uniforms_buffer,
            compiler,
        }
    }

    pub fn upload_uniforms(&self, gpu: &Gpu, uniforms: &ComputeUniforms) {
        uniforms.update_buffer(&self.uniforms_buffer, &gpu)
    }

    pub fn update_shader(&mut self, gpu: &Gpu) {
        let modified_shaders = self.shader_observer.modified();
        if modified_shaders.len() != 0 {
            let mut opts = shaderc::CompileOptions::new().unwrap();
            opts.set_include_callback(|src, _, _, _| {
                Ok(shaderc::ResolvedInclude {
                    resolved_name: format!("./assets/shaders/{}", src),
                    content: fs::read_to_string(format!("./assets/shaders/{}", src)).unwrap(),
                })
            });
            opts.set_optimization_level(shaderc::OptimizationLevel::Performance);

            for shader_path in modified_shaders {
                let shader_name = shader_path.file_name().unwrap().to_str().unwrap();

                let binary_output = self
                    .compiler
                    .compile_into_spirv(
                        fs::read_to_string(shader_path).unwrap().as_str(),
                        shaderc::ShaderKind::Compute,
                        shader_name,
                        "main",
                        Some(&opts),
                    )
                    .unwrap();

                fs::write(
                    format!("./assets/compiled_shaders/{}.spv", shader_name),
                    binary_output.as_binary_u8(),
                )
                .unwrap()
            }

            let shader_module =
                load_spirv_shader("./assets/compiled_shaders/main.glsl.spv", &gpu.device).unwrap();

            self.pipeline = gpu
                .device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("main_compute_pipeline"),
                    module: &shader_module,
                    entry_point: "main",
                    layout: Some(&self.pipeline_layout),
                });
        }
    }

    pub fn execute(&self, gpu: &Gpu, workgroup_size: (u32, u32)) {
        let mut compute_encoder =
            gpu.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("compute_encoder"),
                });

        {
            let mut cpass = compute_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("main_compute_pass"),
            });

            cpass.set_pipeline(&self.pipeline);
            cpass.set_bind_group(0, &self.bind_group, &[]);
            cpass.dispatch(workgroup_size.0, workgroup_size.1, 1);
        }

        gpu.queue.submit(std::iter::once(compute_encoder.finish()));
    }
}
