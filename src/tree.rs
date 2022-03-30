#![allow(clippy::new_without_default)]

use std::rc::Rc;

use crate::render::Mesh;

pub struct Tree {
    roots: [RootQuad; 6],
}

impl Tree {
    pub fn new() -> Self {}

    pub fn collect_meshes(&self, meshes: &mut Vec<Rc<Mesh>>) {
        for root in self.roots {
            root.quad.collect_meshes(meshes);
        }
    }
}

struct RootQuad {
    facing: Facing,
    quad: Quad,
}

enum Quad {
    Leaf(Rc<Mesh>),
    Branch(Box<[Self; 4]>),
}

impl Quad {
    fn collect_meshes(&self, meshes: &mut Vec<Rc<Mesh>>) {
        match self {
            Quad::Leaf(mesh) => meshes.push(mesh.clone()),
            Quad::Branch(children) => {
                for child in children.iter() {
                    child.collect_meshes(meshes);
                }
            }
        }
    }
}

pub enum Facing {
    North,
    South,
    East,
    West,
    Up,
    Down,
}
