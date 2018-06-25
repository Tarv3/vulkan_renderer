use math::{Point, Vec3};

pub struct Ray3 {
    origin: Point,
    dir: Vec3,
}

impl Ray3 {
    pub fn new(origin: Point, dir: Vec3) -> Self {
        Self {
            origin,
            dir
        }
    }
}