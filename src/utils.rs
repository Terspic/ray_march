use crate::camera::Camera;
use wgpu_sandbox::prelude::{
    wgpu::{self, util::DeviceExt},
    Gpu,
};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ComputeUniforms {
    pub camera: Camera,
    pub time: f32,
}

impl ComputeUniforms {
    pub fn new(camera: Camera, time: f32) -> Self {
        Self { camera, time }
    }

    pub fn build_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("compute_uniforms"),
            contents: bytemuck::cast_slice(&[self.clone()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }

    pub fn update_time(&mut self, time: f32) {
        self.time = time;
    }

    pub fn update_camera(&mut self, camera: Camera) {
        self.camera = camera;
    }

    pub fn update_buffer(&self, buffer: &wgpu::Buffer, gpu: &Gpu) {
        gpu.queue
            .write_buffer(buffer, 0, bytemuck::cast_slice(&[self.clone()]))
    }
}

impl Default for ComputeUniforms {
    fn default() -> Self {
        Self {
            camera: Camera::default(),
            time: 0.0,
        }
    }
}

pub fn load_spirv_shader(path: &str, device: &wgpu::Device) -> std::io::Result<wgpu::ShaderModule> {
    let data = std::fs::read(path)?;
    let shader_source = wgpu::ShaderSource::SpirV(wgpu::util::make_spirv_raw(&data));
    Ok(device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: Some(path),
        source: shader_source,
    }))
}
