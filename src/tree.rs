#![allow(clippy::new_without_default)]

use std::rc::Rc;

use cgmath::{vec2, Vector2, Zero};

use crate::{
    camera::Camera,
    gen::{self, PointSampler, QuadInfo, SCALE},
    render::{Mesh, State},
};

pub struct Tree {
    roots: [RootQuad; 6],
}

impl Tree {
    pub fn new(renderer: &State) -> Self {
        Self {
            roots: Facing::all().map(|facing| RootQuad {
                facing,
                quad: Quad::Leaf(gen::quad_mesh(
                    gen::Info {
                        facing,
                        scale: 1.0,
                        offset: Vector2::zero(),
                    },
                    renderer,
                )),
            }),
        }
    }

    pub fn collect_meshes(&self, meshes: &mut Vec<Rc<Mesh>>) {
        for root in &self.roots {
            root.quad.collect_meshes(meshes);
        }
    }

    pub fn process(&mut self, camera: &Camera, renderer: &State) {
        for root in &mut self.roots {
            root.quad.process(
                camera,
                renderer,
                ProcessInfo {
                    facing: root.facing,
                    offset: Vector2::zero(),
                    scale: 1.0,
                },
            );
        }
    }
}

struct RootQuad {
    facing: Facing,
    quad: Quad,
}

enum Quad {
    Leaf(QuadInfo),
    Branch(Box<[Self; 4]>, PointSampler),
}

impl Quad {
    fn collect_meshes(&self, meshes: &mut Vec<Rc<Mesh>>) {
        match self {
            Quad::Leaf(info) => meshes.push(info.mesh.clone()),
            Quad::Branch(children, _) => {
                for child in children.iter() {
                    child.collect_meshes(meshes);
                }
            }
        }
    }

    fn process(&mut self, camera: &Camera, renderer: &State, info: ProcessInfo) {
        match self {
            Quad::Leaf(qinfo) => {
                let min_dist = (info.scale * info.scale) * SCALE;
                let dist = qinfo.sampler.distance2(camera.position());

                if dist < min_dist {
                    let mut sampler = PointSampler::empty();
                    std::mem::swap(&mut sampler, &mut qinfo.sampler);
                    self.subdivide(renderer, info, sampler);
                }
            }
            Quad::Branch(children, sampler) => {
                let max_dist = (info.scale * info.scale) * SCALE * 1.5;
                let dist = sampler.distance2(camera.position());

                if dist > max_dist {
                    self.collapse(renderer, info);
                    return;
                }

                let offsets = [
                    vec2(-1.0, 1.0),
                    vec2(1.0, 1.0),
                    vec2(-1.0, -1.0),
                    vec2(1.0, -1.0),
                ];

                for (child, offset) in children.iter_mut().zip(offsets.into_iter()) {
                    child.process(
                        camera,
                        renderer,
                        ProcessInfo {
                            facing: info.facing,
                            offset: info.offset + offset * info.scale * 0.5,
                            scale: info.scale * 0.5,
                        },
                    );
                }
            }
        }
    }

    fn subdivide(&mut self, renderer: &State, info: ProcessInfo, sampler: PointSampler) {
        if matches!(self, Self::Leaf(_)) {
            let scale = info.scale * 0.5;
            *self = Self::Branch(
                Box::new([
                    Quad::Leaf(gen::quad_mesh(
                        gen::Info {
                            facing: info.facing,
                            scale,
                            offset: info.offset + vec2(-1.0, 1.0) * scale,
                        },
                        renderer,
                    )),
                    Quad::Leaf(gen::quad_mesh(
                        gen::Info {
                            facing: info.facing,
                            scale,
                            offset: info.offset + vec2(1.0, 1.0) * scale,
                        },
                        renderer,
                    )),
                    Quad::Leaf(gen::quad_mesh(
                        gen::Info {
                            facing: info.facing,
                            scale,
                            offset: info.offset + vec2(-1.0, -1.0) * scale,
                        },
                        renderer,
                    )),
                    Quad::Leaf(gen::quad_mesh(
                        gen::Info {
                            facing: info.facing,
                            scale,
                            offset: info.offset + vec2(1.0, -1.0) * scale,
                        },
                        renderer,
                    )),
                ]),
                sampler,
            );
        }
    }

    fn collapse(&mut self, renderer: &State, info: ProcessInfo) {
        if matches!(self, Self::Branch(_, _)) {
            *self = Self::Leaf(gen::quad_mesh(
                gen::Info {
                    facing: info.facing,
                    scale: info.scale,
                    offset: info.offset,
                },
                renderer,
            ))
        }
    }
}

#[derive(Clone, Copy)]
pub enum Facing {
    North,
    South,
    East,
    West,
    Up,
    Down,
}

impl Facing {
    fn all() -> [Self; 6] {
        [
            Self::North,
            Self::South,
            Self::East,
            Self::West,
            Self::Up,
            Self::Down,
        ]
    }
}

struct ProcessInfo {
    facing: Facing,
    offset: Vector2<f32>,
    scale: f32,
}
