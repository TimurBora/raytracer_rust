use crate::Material;
use crate::Vec3f;

pub trait Intersectable {
    fn ray_intersect(&self, origin: Vec3f, direction: Vec3f) -> Option<f64>;
}

pub trait Shape: Intersectable {
    fn get_normal(&self, hit_point: Vec3f) -> Vec3f;
    fn get_material(&self) -> Material;
}

#[derive(Clone)]
pub struct Sphere {
    center: Vec3f,
    radius: f64,
    material: Material,
}

impl Sphere {
    pub fn new(center: Vec3f, radius: f64, material: Material) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }
}

impl Intersectable for Sphere {
    fn ray_intersect(&self, origin: Vec3f, direction: Vec3f) -> Option<f64> {
        let l = self.center - origin;
        let tca = l * direction;
        let d2 = l * l - tca * tca;

        if d2 > self.radius * self.radius {
            return None;
        }

        let thc = (self.radius * self.radius - d2).sqrt();
        let mut t0 = tca - thc;
        let t1 = tca + thc;

        if t0 < 0.0 {
            t0 = t1;
        }
        if t0 < 0.0 {
            return None;
        }

        Some(t0)
    }
}

impl Shape for Sphere {
    fn get_material(&self) -> Material {
        self.material
    }

    fn get_normal(&self, hit_point: Vec3f) -> Vec3f {
        (hit_point - self.center).normalize(None)
    }
}
