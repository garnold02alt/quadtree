struct Attribs {
    [[location(0)]]
    position: vec3<f32>;

    [[location(1)]]
    normal: vec3<f32>;
};

struct Vertex {
    [[builtin(position)]]
    position: vec4<f32>;

    [[location(0)]]
    normal: vec3<f32>;

    [[location(1)]]
    world_position: vec3<f32>;

    [[location(2)]]
    camera_position: vec3<f32>;
};

struct Camera {
    world_to_clip: mat4x4<f32>;
    view_to_world: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> camera: Camera;

[[stage(vertex)]]
fn vertex(attribs: Attribs) -> Vertex {
    var vertex: Vertex;
    vertex.position = camera.world_to_clip * vec4<f32>(attribs.position, 1.0);
    vertex.normal = attribs.normal;
    vertex.camera_position = (camera.view_to_world * vec4<f32>(0.0, 0.0, 0.0, 1.0)).xyz;
    vertex.world_position = attribs.position;
    return vertex;
}

struct Fragment {
    [[location(0)]]
    color: vec4<f32>;
};

[[stage(fragment)]]
fn fragment(vertex: Vertex) -> Fragment {
    var light_dir = normalize(vertex.camera_position - vertex.world_position);
    var light_intensity = max(dot(light_dir, vertex.normal), 0.0);

    var fragment: Fragment;
    fragment.color = vec4<f32>(light_intensity + 0.1);
    return fragment;
}