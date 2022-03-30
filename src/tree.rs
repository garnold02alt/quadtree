#![allow(clippy::new_without_default)]

pub struct QuadTree {
    leaves: [Leaf; 6],
}

enum Leaf {
    Filled,
    Subdivided(Box<[Self; 4]>),
}

impl QuadTree {
    pub fn new() -> Self {
        Self {
            leaves: [
                Leaf::Filled,
                Leaf::Filled,
                Leaf::Filled,
                Leaf::Filled,
                Leaf::Filled,
                Leaf::Filled,
            ],
        }
    }
}
