@group(0) @binding(0)
var<storage, read_write> output: array<u32>;

@group(0) @binding(1) var<uniform> camera: CameraUniform;

struct Ray {
    origin: point3,
    direction: vec3f,
}

struct CameraUniform {
    camera_center: vec3<f32>,
    focal_length: f32,
    viewport_height: f32,
}

fn ray_color(r: Ray) -> color {
    let unit_direction: vec3f = normalize(r.direction);
    let a = 0.5 * (unit_direction.y + 1.0);

    // Lerp:
    var pixel_color = (1.0 - a) * color(1.0, 1.0, 1.0) + a * color(0.1, 0.3, 1.0);

    let t = hit_sphere(point3(0, 0, -1), 0.5, r);
    if t > 0.0 {
        let N: vec3f = normalize(ray_at(r, t) - vec3(0, 0, -1));
        return 0.5 * color(N.x + 1, N.y + 1, N.z + 1);
    }

    return pixel_color;
}

fn hit_sphere(center: vec3<f32>, radius: f32, r: Ray) -> f32 {
    let oc: vec3f = center - r.origin;
    let a: f32 = dot(r.direction, r.direction);
    let b: f32 = -2.0 * dot(r.direction, oc);
    let c: f32 = dot(oc, oc) - radius * radius;
    let discriminant: f32 = b * b - 4.0 * a * c;

    if discriminant < 0 {
        return -1.0;
    } else {
        return (-b - sqrt(discriminant)) / (2.0 * a);
    }
}

// TODO: Move Image specs to rust code and allow for user input
// ====================  Image  ===================== //
const aspect_ratio: f32 = 2.0 / 1.0;
const image_width: u32 = 1024u;
const image_height: u32 = u32(f32(image_width) / aspect_ratio);
// ====================  /Image  ==================== //

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // Camera parameters
    let camera_center = camera.camera_center;
    let focal_length = camera.focal_length;
    let viewport_height = camera.viewport_height;
    let viewport_width: f32 = viewport_height * (f32(image_width) / f32(image_height));
    let viewport_u: vec3f = vec3(viewport_width, 0, 0);
    let viewport_v: vec3f = vec3(0, -viewport_height, 0);
    let pixel_delta_u = viewport_u / f32(image_width);
    let pixel_delta_v = viewport_v / f32(image_height);
    let viewport_upper_left = camera_center - vec3(0, 0, focal_length) - viewport_u / 2 - viewport_v / 2;
    let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);


    let pixel_center: vec3f = pixel00_loc + (f32(global_id.x) * pixel_delta_u) + (f32(global_id.y) * pixel_delta_v);
    let ray_direction: vec3f = pixel_center - camera_center;
    let r: Ray = Ray(camera_center, ray_direction);

    let pixel_color: color = ray_color(r);

    let packed: u32 = get_packed_rgba_color(pixel_color);
    output[global_id.y * image_width + global_id.x] = packed;
}

// Helpers & aliases
alias point3 = vec3<f32>;
alias color = vec3<f32>;

fn get_packed_rgba_color(c: color) -> u32 {
    let r = u32(c.r * 255.0);
    let g = u32(c.g * 255.0);
    let b = u32(c.b * 255.0);
    return (r | (g << 8u) | (b << 16u) | (255u << 24u));
}

fn ray_at(r: Ray, t: f32) -> vec3<f32> {
    return r.origin + t * r.direction;
}
