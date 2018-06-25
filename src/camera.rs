use math;
use math::{Isometry, Mat4, Orthographic, Perspective, Point, Projection, Rotation, Vec3};
use nalgebra::Unit;
use ray::Ray3;
use std::f32::consts::PI;

pub struct Camera<T: Projection> {
    eye: Point,
    target: Point,
    up: Unit<Vec3>,
    projection: T,
}

impl<T: Projection> Camera<T> {
    pub fn new(eye: Point, target: Point, up: Vec3, projection: T) -> Self {
        Self {
            eye,
            target,
            up: Unit::new_normalize(up),
            projection,
        }
    }
    pub fn move_eye_to(&mut self, new_pos: Point) {
        self.eye = new_pos;
    }
    pub fn move_eye_by(&mut self, vec: Vec3) {
        self.eye += vec;
    }
    pub fn move_target_by(&mut self, vec: Vec3) {
        self.target += vec;
    }
    pub fn move_eye_and_target_by(&mut self, vec: Vec3) {
        self.move_eye_by(vec);
        self.move_target_by(vec);
    }
    // Rotates target around the up direction
    pub fn rotate_target_around_up(&mut self, angle: f32) {
        self.target -= self.eye.coords;
        let rotation = Rotation::from_axis_angle(&self.up, angle);
        self.target = rotation * self.target + self.eye.coords;
    }
    // Rotates the target around the right direction
    pub fn rotate_target_around_right(&mut self, angle: f32) {
        self.target -= self.eye.coords;
        let rotation = Rotation::from_axis_angle(&self.right_dir(), angle);
        self.target = rotation * self.target + self.eye.coords;
    }
    pub fn up_dir(&self) -> Unit<Vec3> {
        self.up
    }
    // The cross product of the look direction and the up direction
    pub fn right_dir(&self) -> Unit<Vec3> {
        let look_dir = self.target - self.eye;
        let right = math::cross(look_dir, self.up.unwrap());
        Unit::new_normalize(right)
    }
    // The normalised eye to target vector 
    pub fn look_dir(&self) -> Unit<Vec3> {
        Unit::new_normalize(self.target - self.eye)
    }
    pub fn look_at(&mut self, new_look: Point) {
        self.target = new_look;
    }
    pub fn look_at_matrix(&self) -> Isometry {
        Isometry::look_at_rh(&self.eye, &self.target, &self.up)
    }
    // The matrix representing the projection matrix multiplied by the view matrix
    pub fn view_projection(&self) -> Mat4 {
        self.projection.matrix() * self.look_at_matrix().to_homogeneous()
    }
    pub fn set_znear(&mut self, znear: f32) {
        self.projection.set_znear(znear);
    }
    pub fn set_zfar(&mut self, zfar: f32) {
        self.projection.set_zfar(zfar);
    }
    pub fn projection_ref(&self) -> &T {
        &self.projection
    }
    // Note: x and y must range from -1 to 1
    pub fn screen_to_world_space(&self, x: f32, y: f32) -> Ray3 {
        let point = Point::new(x, y, 1.0);
        let point = self.projection.unproject(point);
        let point = self.look_at_matrix().inverse() * point;
        let dir = point - self.eye;
        Ray3::new(self.eye, dir)
    }
}

impl Default for Camera<Perspective> {
    fn default() -> Self {
        Self::new(
            Point::new(0.0, 0.0, 0.0),
            Point::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 1.0, 0.0),
            Perspective::new(1.777778, PI * 0.5, 0.1, 1000.0),
        )
    }
}

impl Default for Camera<Orthographic> {
    fn default() -> Self {
        Self::new(
            Point::new(0.0, 0.0, 0.0),
            Point::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 1.0, 0.0),
            Orthographic::new(-1.0, 1.0, -1.0, 1.0, 0.1, 1000.0),
        )
    }
}

impl Camera<Perspective> {
    pub fn set_fovy(&mut self, new_fovy: f32) {
        self.projection.set_fovy(new_fovy);
    }
    pub fn set_aspect(&mut self, new_aspect: f32) {
        self.projection.set_aspect(new_aspect);
    }
    pub fn get_fovy(&self) -> f32 {
        self.projection.fovy()
    }
    pub fn get_aspect(&self) -> f32 {
        self.projection.aspect()
    }
}
