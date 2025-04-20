use crate::EPSILON;
use crate::Material;
use crate::Vec3f;

pub trait Intersectable {
    fn ray_intersect(&self, origin: Vec3f, direction: Vec3f) -> Option<f64>;
}

pub trait Shape: Intersectable {
    fn get_normal(&self, hit_point: Vec3f) -> Vec3f;
    fn get_material(&self) -> Material;
}

#[derive(Clone, Debug)]
pub struct Sphere {
    center: Vec3f,
    radius: f64,
    material: Material,
}

impl Sphere {
    #[allow(dead_code)]
    pub const fn new(center: Vec3f, radius: f64, material: Material) -> Self {
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
        let d2 = tca.mul_add(-tca, l * l);

        if d2 > self.radius * self.radius {
            return None;
        }

        let thc = self.radius.mul_add(self.radius, -d2).sqrt();
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

#[derive(Clone, Debug)]
pub struct BoxShape {
    max_point: Vec3f,
    min_point: Vec3f,
    material: Material,
}

impl BoxShape {
    #[allow(dead_code)]
    pub const fn new(max_point: Vec3f, min_point: Vec3f, material: Material) -> Self {
        Self {
            max_point,
            min_point,
            material,
        }
    }
}

impl Intersectable for BoxShape {
    fn ray_intersect(&self, origin: Vec3f, direction: Vec3f) -> Option<f64> {
        let inv_dir = Vec3f::const_new_with_data([
            1.0 / direction.x(),
            1.0 / direction.y(),
            1.0 / direction.z(),
        ]);

        let t1 = (self.min_point.x() - origin.x()) * inv_dir.x();
        let t2 = (self.max_point.x() - origin.x()) * inv_dir.x();
        let t3 = (self.min_point.y() - origin.y()) * inv_dir.y();
        let t4 = (self.max_point.y() - origin.y()) * inv_dir.y();
        let t5 = (self.min_point.z() - origin.z()) * inv_dir.z();
        let t6 = (self.max_point.z() - origin.z()) * inv_dir.z();

        let tmin = f64::max(
            f64::max(f64::min(t1, t2), f64::min(t3, t4)),
            f64::min(t5, t6),
        );
        let tmax = f64::min(
            f64::min(f64::max(t1, t2), f64::max(t3, t4)),
            f64::max(t5, t6),
        );

        if tmax < 0.0 || tmin > tmax {
            return None;
        }

        Some(if tmin < 0.0 { tmax } else { tmin })
    }
}

impl Shape for BoxShape {
    fn get_material(&self) -> Material {
        self.material
    }
    fn get_normal(&self, hit_point: Vec3f) -> Vec3f {
        let mut normal = Vec3f::new_with_data([0.0, 0.0, 0.0]);

        if f64::abs(hit_point.x() - self.min_point.x()) < EPSILON {
            normal = Vec3f::new_with_data([-1.0, 0.0, 0.0]); // левая грань
        } else if f64::abs(hit_point.x() - self.max_point.x()) < EPSILON {
            normal = Vec3f::new_with_data([1.0, 0.0, 0.0]); // правая грань
        } else if f64::abs(hit_point.y() - self.min_point.y()) < EPSILON {
            normal = Vec3f::new_with_data([0.0, -1.0, 0.0]); // нижняя грань
        } else if f64::abs(hit_point.y() - self.max_point.y()) < EPSILON {
            normal = Vec3f::new_with_data([0.0, 1.0, 0.0]); // верхняя грань
        } else if f64::abs(hit_point.z() - self.min_point.z()) < EPSILON {
            normal = Vec3f::new_with_data([0.0, 0.0, -1.0]); // задняя грань
        } else if f64::abs(hit_point.z() - self.max_point.z()) < EPSILON {
            normal = Vec3f::new_with_data([0.0, 0.0, 1.0]); // передняя грань
        }

        normal
    }
}

#[derive(Clone, Debug)]
pub struct InfinityPlane {
    position: Vec3f,
    normal: Vec3f,
    material: Material,
}

impl InfinityPlane {
    #[allow(dead_code)]
    pub fn new(position: Vec3f, normal: Vec3f, material: Material) -> Self {
        Self {
            position,
            normal: normal.normalize(None),
            material,
        }
    }
}

impl Shape for InfinityPlane {
    fn get_material(&self) -> Material {
        self.material
    }

    fn get_normal(&self, _hit_point: Vec3f) -> Vec3f {
        self.normal
    }
}

impl Intersectable for InfinityPlane {
    fn ray_intersect(&self, origin: Vec3f, direction: Vec3f) -> Option<f64> {
        let ray_point = direction * self.normal;
        if f64::abs(ray_point) < EPSILON {
            return None;
        }

        let s = (self.normal * (self.position - origin)) / ray_point;
        if s < 0.0 {
            return None;
        }

        Some(s)
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum ShapeType {
    Sphere(Sphere),
    BoxShape(BoxShape),
    InfinityPlane(InfinityPlane),
}

impl Shape for ShapeType {
    fn get_material(&self) -> Material {
        match self {
            Self::Sphere(sphere) => sphere.get_material(),
            Self::BoxShape(box_shape) => box_shape.get_material(),
            Self::InfinityPlane(plane) => plane.get_material(),
        }
    }

    fn get_normal(&self, hit_point: Vec3f) -> Vec3f {
        match self {
            Self::Sphere(sphere) => sphere.get_normal(hit_point),
            Self::BoxShape(box_shape) => box_shape.get_normal(hit_point),
            Self::InfinityPlane(plane) => plane.get_normal(hit_point),
        }
    }
}

impl Intersectable for ShapeType {
    fn ray_intersect(&self, origin: Vec3f, direction: Vec3f) -> Option<f64> {
        match self {
            Self::Sphere(sphere) => sphere.ray_intersect(origin, direction),
            Self::BoxShape(box_shape) => box_shape.ray_intersect(origin, direction),
            Self::InfinityPlane(plane) => plane.ray_intersect(origin, direction),
        }
    }
}
