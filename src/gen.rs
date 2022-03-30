#![allow(clippy::needless_range_loop)]
#![allow(clippy::new_without_default)]

use cgmath::{vec2, vec3, Vector2, Vector3, Zero};

use crate::render::{Mesh, State, Vertex};

const LATTICE_LEN: usize = 16;
const P_COUNT: usize = LATTICE_LEN + 1;
const T_COUNT: usize = LATTICE_LEN * LATTICE_LEN * 2;

pub struct Lattice {
    points: [[Vector3<f32>; P_COUNT]; P_COUNT],
    triangles: [[u16; 3]; T_COUNT],
}

impl Lattice {
    pub fn new(face: Face) -> Self {
        let scalar = 1.0 / P_COUNT as f32;
        let mut points = [[Vector3::zero(); P_COUNT]; P_COUNT];

        for y in 0..P_COUNT {
            for x in 0..P_COUNT {
                let point = &mut points[y][x];
                let flat = vec2(x as f32, y as f32).map(|e| e * scalar - 0.5);
                *point = face.orient(flat);
            }
        }

        let mut triangles = [[0u16; 3]; T_COUNT];

        for y in 0..LATTICE_LEN {
            for x in 0..LATTICE_LEN {
                let i0 = (y * P_COUNT + x) as u16;
                let i1 = (y * P_COUNT + x + 1) as u16;
                let i2 = ((y + 1) * P_COUNT + x + 1) as u16;
                let i3 = ((y + 1) * P_COUNT + x) as u16;

                triangles[(y * LATTICE_LEN + x) * 2] = [i0, i1, i2];
                triangles[(y * LATTICE_LEN + x) * 2 + 1] = [i0, i2, i3];
            }
        }

        Self { points, triangles }
    }

    pub fn into_mesh(self, renderer: &State) -> Mesh {
        let vertices = self
            .points
            .into_iter()
            .flatten()
            .map(|point| Vertex {
                position: point,
                normal: vec3(0.0, 0.0, 1.0),
            })
            .collect::<Vec<_>>();

        renderer.create_mesh(&vertices, &self.triangles)
    }
}

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
            Self::North => vec3(flat.x, flat.y, -1.0),
            Self::South => vec3(-flat.x, flat.y, 1.0),
            Self::East => vec3(-1.0, flat.y, flat.x),
            Self::West => vec3(1.0, flat.y, -flat.x),
            Self::Top => vec3(-flat.x, 1.0, flat.y),
            Self::Bottom => vec3(flat.x, -1.0, flat.y),
        }
    }
}
