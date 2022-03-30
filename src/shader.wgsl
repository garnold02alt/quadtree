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
};

struct Camera {
    matrix: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> camera: Camera;

[[stage(vertex)]]
fn vertex(attribs: Attribs) -> Vertex {
    var vertex: Vertex;
    vertex.position = camera.matrix * vec4<f32>(attribs.position, 1.0);
    vertex.normal = attribs.normal;
    return vertex;
}

struct Fragment {
    [[location(0)]]
    color: vec4<f32>;
};

[[stage(fragment)]]
fn fragment(vertex: Vertex) -> Fragment {
    var fragment: Fragment;
    fragment.color = vec4<f32>(1.0);
    return fragment;
}