mod renderer;
mod scene;
mod camera;

fn main() {
    env_logger::init();
    // TODO: Use the triangles
    let _triangles = scene::load();
    pollster::block_on(renderer::run());
}
