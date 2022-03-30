use std::rc::Rc;

use cgmath::{vec2, vec3, InnerSpace, MetricSpace, Vector2, Vector3, Zero};
use noise::{NoiseFn, SuperSimplex};
use rand::Rng;

use crate::{
    render::{Mesh, State, Vertex},
    tree::Facing,
};

pub const SCALE: f32 = 100.0;

const L_QUADS: usize = 32;
const L_POINTS: usize = L_QUADS + 1;
const FLAT_SCALAR: f32 = 2.0 / L_QUADS as f32;

pub fn quad_mesh(info: Info, renderer: &State) -> QuadInfo {
    let mut rng = rand::thread_rng();
    let noise = ElevationSampler::new(4, 2.0, 2.5, 0.1, 0.3);

    let color: Vector3<f32> = From::<[f32; 3]>::from(rng.gen());
    let mut vertices = Vec::with_capacity(L_POINTS * L_POINTS);
    let mut triangles = Vec::with_capacity(L_QUADS * L_QUADS * 2);
    let mut points = Vec::new();

    for y in 0..L_POINTS {
        for x in 0..L_POINTS {
            let flat = vec2(x as f32, y as f32).map(|e| e * FLAT_SCALAR - 1.0);
            let offset_scaled = flat * info.scale + info.offset;
            let oriented = info.facing.orient(offset_scaled);
            let normalized = oriented.normalize();

            let elevation = noise.sample(normalized);
            let position = normalized * SCALE + normalized * elevation * SCALE;

            points.push(position);
            vertices.push(Vertex {
                position,
                normal: Vector3::zero(),
                color,
            });
        }
    }

    for y in 0..L_QUADS {
        for x in 0..L_QUADS {
            let i0 = (y * L_POINTS + x) as u16;
            let i1 = (y * L_POINTS + x + 1) as u16;
            let i2 = ((y + 1) * L_POINTS + x + 1) as u16;
            let i3 = ((y + 1) * L_POINTS + x) as u16;

            triangles.push([i0, i1, i2]);
            triangles.push([i0, i2, i3]);
        }
    }

    for y in 0..L_QUADS {
        for x in 0..L_QUADS {
            let p0 = vertices[y * L_POINTS + x].position;
            let p1 = vertices[y * L_POINTS + x + 1].position;
            let p2 = vertices[(y + 1) * L_POINTS + x].position;
            let e0 = p1 - p0;
            let e1 = p2 - p0;
            let normal = e0.cross(e1).normalize();

            vertices[y * L_POINTS + x].normal += normal;
            vertices[y * L_POINTS + x + 1].normal += normal;
            vertices[(y + 1) * L_POINTS + x + 1].normal += normal;
            vertices[(y + 1) * L_POINTS + x].normal += normal;
        }
    }

    for vertex in &mut vertices {
        vertex.normal = vertex.normal.normalize();
    }

    QuadInfo {
        mesh: Rc::new(renderer.create_mesh(&vertices, &triangles)),
        sampler: PointSampler { points },
    }
}

pub struct Info {
    pub facing: Facing,
    pub scale: f32,
    pub offset: Vector2<f32>,
}

impl Facing {
    fn orient(&self, vec: Vector2<f32>) -> Vector3<f32> {
        match self {
            Self::North => vec3(-vec.x, vec.y, -1.0),
            Self::South => vec3(vec.x, vec.y, 1.0),
            Self::East => vec3(-1.0, vec.y, vec.x),
            Self::West => vec3(1.0, vec.y, -vec.x),
            Self::Up => vec3(-vec.x, 1.0, vec.y),
            Self::Down => vec3(vec.x, -1.0, vec.y),
        }
    }
}

pub struct QuadInfo {
    pub mesh: Rc<Mesh>,
    pub sampler: PointSampler,
}

pub struct PointSampler {
    points: Vec<Vector3<f32>>,
}

impl PointSampler {
    pub fn distance2(&self, point: Vector3<f32>) -> f32 {
        let mut shortest = f32::INFINITY;

        for other in &self.points {
            let dist = point.distance2(*other);
            if dist < shortest {
                shortest = dist;
            }
        }

        shortest
    }

    pub fn empty() -> Self {
        Self { points: Vec::new() }
    }
}

struct ElevationSampler {
    noise: SuperSimplex,
    octaves: u32,

    init_freq: f32,
    delta_freq: f32,

    init_ampl: f32,
    delta_ampl: f32,
}

impl ElevationSampler {
    pub fn new(oct: u32, fi: f32, fd: f32, ai: f32, ad: f32) -> Self {
        Self {
            noise: SuperSimplex::new(),
            octaves: oct,
            init_freq: fi,
            delta_freq: fd,
            init_ampl: ai,
            delta_ampl: ad,
        }
    }

    pub fn sample(&self, position: Vector3<f32>) -> f32 {
        let mut freq = self.init_freq;
        let mut ampl = self.init_ampl;
        let mut value = 0.0;

        for _ in 0..self.octaves {
            let coords: [f64; 3] = (position * freq).cast().unwrap().into();
            let sample = self.noise.get(coords) as f32;
            value += sample * ampl;

            freq *= self.delta_freq;
            ampl *= self.delta_ampl;
        }

        value
    }
}
