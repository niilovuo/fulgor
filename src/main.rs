mod camera;
mod renderer;
mod scene;

fn main() {
    env_logger::init();
    // TODO: Use the triangles
    let _triangles = scene::load();

    let camera_uniform = camera::CameraUniform {
        camera_center: [0.0, 0.0, 0.0],
        focal_length: 1.0,
        viewport_height: 2.0,
        _pad: [0.0, 0.0, 0.0],
    };

    pollster::block_on(renderer::run(camera_uniform));
}
