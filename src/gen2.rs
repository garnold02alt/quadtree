#![allow(clippy::needless_range_loop)]
#![allow(clippy::new_without_default)]

use cgmath::{vec2, vec3, InnerSpace, Vector2, Vector3, Zero};
use noise::{NoiseFn, OpenSimplex};
use rand::Rng;

use crate::render::{Mesh, State, Vertex};

const LATTICE_LEN: usize = 16;
const P_LEN: usize = LATTICE_LEN + 1;
const T_COUNT: usize = LATTICE_LEN * LATTICE_LEN * 2;

pub struct Lattice {
    points: [[Vector3<f32>; P_LEN]; P_LEN],
    triangles: Vec<[u16; 3]>,
}

impl Lattice {
    pub fn new(face: Face, level: i32, disp: Vector2<f32>) -> Self {
        let scalar = 1.0 / (P_LEN - 1) as f32;
        let mut points = [[Vector3::zero(); P_LEN]; P_LEN];

        for y in 0..P_LEN {
            for x in 0..P_LEN {
                let point = &mut points[y][x];
                let flat = vec2(x as f32, y as f32).map(|e| (e * scalar - 0.5) * 2.0);
                let level_scaled = flat / 2.0f32.powi(level);
                let level_disp = level_scaled + disp;
                *point = face.orient(level_disp);
            }
        }

        let mut triangles = Vec::with_capacity(T_COUNT);

        for y in 0..LATTICE_LEN {
            for x in 0..LATTICE_LEN {
                let i0 = (y * P_LEN + x) as u16;
                let i1 = (y * P_LEN + x + 1) as u16;
                let i2 = ((y + 1) * P_LEN + x + 1) as u16;
                let i3 = ((y + 1) * P_LEN + x) as u16;

                triangles.push([i0, i1, i2]);
                triangles.push([i0, i2, i3]);
            }
        }

        Self { points, triangles }
    }

    pub fn into_quad(self) -> QuadMesh {
        let noise = OpenSimplex::new();
        let color: [f32; 3] = rand::thread_rng().gen();
        let color: Vector3<f32> = color.into();

        let mut vertices = self
            .points
            .into_iter()
            .flatten()
            .map(|point| {
                let spherical = point.normalize();

                Vertex {
                    position: mkpoint(spherical, &noise).cast().unwrap(),
                    normal: Vector3::zero(),
                    color,
                }
            })
            .collect::<Vec<_>>();

        for y in 0..LATTICE_LEN {
            for x in 0..LATTICE_LEN {
                let p0 = vertices[y * P_LEN + x].position;
                let p1 = vertices[y * P_LEN + x + 1].position;
                let p2 = vertices[(y + 1) * P_LEN + x].position;
                let e0 = p1 - p0;
                let e1 = p2 - p0;
                let normal = e0.cross(e1).normalize();

                vertices[y * P_LEN + x].normal += normal;
                vertices[y * P_LEN + x + 1].normal += normal;
                vertices[(y + 1) * P_LEN + x + 1].normal += normal;
                vertices[(y + 1) * P_LEN + x].normal += normal;
            }
        }

        for vertex in &mut vertices {
            vertex.normal = vertex.normal.normalize();
        }

        fn mkpoint(sph: Vector3<f32>, noise: &OpenSimplex) -> Vector3<f32> {
            let xyz: [f64; 3] = (sph * 10.0).cast::<f64>().unwrap().into();
            let elevation = noise.get(xyz) as f32;
            sph * (1.0 + elevation * 0.0)
        }

        QuadMesh {
            vertices,
            triangles: self.triangles,
        }
    }
}

#[derive(Clone, Copy)]
pub enum Face {
    North,
    South,
    East,
    West,
    Top,
    Bottom,
}

impl Face {
    fn orient(&self, flat: Vector2<f32>) -> Vector3<f32> {
        match self {
            Self::North => vec3(-flat.x, flat.y, -1.0),
            Self::South => vec3(flat.x, flat.y, 1.0),
            Self::East => vec3(-1.0, flat.y, flat.x),
            Self::West => vec3(1.0, flat.y, -flat.x),
            Self::Top => vec3(-flat.x, 1.0, flat.y),
            Self::Bottom => vec3(flat.x, -1.0, flat.y),
        }
    }
}

pub struct QuadMesh {
    vertices: Vec<Vertex>,
    triangles: Vec<[u16; 3]>,
}

impl QuadMesh {
    pub fn into_mesh(self, renderer: &State) -> Mesh {
        renderer.create_mesh(&self.vertices, &self.triangles)
    }
}

pub enum Quadrant {
    UL,
    UR,
    DL,
    DR,
}

impl Quadrant {
    fn vector(&self) -> Vector2<f32> {
        match self {
            Self::UL => vec2(-1.0, 1.0),
            Self::UR => vec2(1.0, 1.0),
            Self::DL => vec2(-1.0, -1.0),
            Self::DR => vec2(1.0, -1.0),
        }
    }
}

impl TryFrom<usize> for Quadrant {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Quadrant::UL),
            1 => Ok(Quadrant::UR),
            2 => Ok(Quadrant::DL),
            3 => Ok(Quadrant::DR),
            _ => Err(()),
        }
    }
}

impl TryFrom<usize> for Face {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Face::North),
            1 => Ok(Face::South),
            2 => Ok(Face::East),
            3 => Ok(Face::West),
            4 => Ok(Face::Top),
            5 => Ok(Face::Bottom),
            _ => Err(()),
        }
    }
}

trait Dispalce {
    fn displace(self) -> Vector2<f32>;
}

impl Dispalce for &[Quadrant] {
    fn displace(self) -> Vector2<f32> {
        let mut level = 1;
        let mut disp = Vector2::zero();

        for quad in self {
            disp += quad.vector() / 2.0f32.powi(level);
            level += 1;
        }

        disp
    }
}
