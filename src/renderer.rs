use std::time::SystemTime;

use wgpu::util::DeviceExt;

use crate::camera;

const WIDTH: u32 = 1024;
const HEIGHT: u32 = 512;

pub async fn run(camera_uniform: camera::CameraUniform) {
    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .expect("No GPU adapter found");
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default())
        .await
        .expect("Failed to create device");

    // Get info related to the GPU being used:
    let info = adapter.get_info();
    println!("Using GPU: {} ({:?})", info.name, info.backend);

    let buffer_size = (WIDTH * HEIGHT * 4) as u64;
    let storage_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Output Buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Staging Buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Camera Buffer"),
        contents: bytemuck::cast_slice(&[camera_uniform]),
        usage: wgpu::BufferUsages::UNIFORM,
    });

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/rt.wgsl").into()),
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: storage_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: camera_buffer.as_entire_binding(),
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[Some(&bind_group_layout)],
        immediate_size: 0,
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Compute Pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        let wg = 16;
        pass.dispatch_workgroups(WIDTH.div_ceil(wg), HEIGHT.div_ceil(wg), 1);
    }
    encoder.copy_buffer_to_buffer(&storage_buffer, 0, &staging_buffer, 0, buffer_size);
    let now = SystemTime::now();
    queue.submit(std::iter::once(encoder.finish()));
    println!("Render time: {}µs", now.elapsed().unwrap().as_micros());

    let slice = staging_buffer.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    let _res = device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: None,
    });

    let data = slice.get_mapped_range();
    let rgba_image =
        image::RgbaImage::from_raw(WIDTH, HEIGHT, data.to_vec()).expect("Buffer size mismatch");
    let rgb_image = image::DynamicImage::ImageRgba8(rgba_image);
    rgb_image.save("output/render.png").unwrap();
    println!("Image saved as render.png");
}
