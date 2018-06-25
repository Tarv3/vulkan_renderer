use nalgebra::{Vector3, Perspective3, Matrix4, Point3, Unit, Orthographic3, Isometry3, Rotation3};


pub type Isometry = Isometry3<f32>;
pub type Orthographic = Orthographic3<f32>;
pub type Perspective = Perspective3<f32>;
pub type Rotation = Rotation3<f32>;
pub type Vec3 = Vector3<f32>;
pub type Mat4 = Matrix4<f32>;
pub type Point = Point3<f32>;
pub type Unit3 = Unit<Vec3>;

impl Projection for Perspective {
    fn matrix(&self) -> &Mat4 {
        self.as_matrix()
    }
    fn set_znear_zfar(&mut self, znear: f32, zfar: f32) {
        self.set_znear_and_zfar(znear, zfar);
    }
    fn unproject(&self, point: Point) -> Point{
        self.unproject_point(&point)
    }
    fn get_znear(&self) -> f32 {
        self.znear()
    }
    fn get_zfar(&self) -> f32 {
        self.zfar()
    }
}

impl Projection for Orthographic {
    fn matrix(&self) -> &Mat4 {
        self.as_matrix()
    }
    fn unproject(&self, point: Point) -> Point{
        self.unproject_point(&point)
    }
    
    fn set_znear_zfar(&mut self, znear: f32, zfar: f32) {
        self.set_znear_and_zfar(znear, zfar);
    }
    fn get_znear(&self) -> f32 {
        self.znear()
    }
    fn get_zfar(&self) -> f32 {
        self.zfar()
    }
}

pub trait Projection {
    fn matrix(&self) -> &Mat4;
    fn set_znear_zfar(&mut self, znear: f32, zfar: f32);
    fn get_znear(&self) -> f32;
    fn get_zfar(&self) -> f32;
    fn unproject(&self, point: Point) -> Point;
    fn set_znear(&mut self, znear: f32) {
        let zfar = self.get_zfar();
        self.set_znear_zfar(znear, zfar)
    }
    fn set_zfar(&mut self, zfar: f32) {
        let znear = self.get_znear();
        self.set_znear_zfar(znear, zfar)
    }
    fn as_slice(&self) -> &[f32] {
        self.matrix().as_slice()
    }
}

pub fn cross(a: Vec3, b: Vec3) -> Vec3 {
    let x = a.y * b.z - a.z * b.y;
    let y = a.z * b.x - a.x * b.z;
    let z = a.x * b.y - a.y * b.x;

    Vec3::new(x, y, z)
}