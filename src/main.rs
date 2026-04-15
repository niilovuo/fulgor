mod camera;
mod renderer;
mod scene;

fn main() {
    env_logger::init();
    // TODO: Use the triangles
    let _triangles = scene::load();
    pollster::block_on(renderer::run());
}
