#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub camera_center: [f32; 3],
    pub focal_length: f32,
    pub viewport_height: f32,
    pub _pad: [f32; 3],
}
