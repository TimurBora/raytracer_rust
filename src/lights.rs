use crate::Vec3f;

pub trait Light {
    fn intensity(&self) -> f64;
    fn get_distance(&self, _point: Vec3f) -> f64;
    fn get_direction(&self, _point: Vec3f) -> Vec3f;
    fn is_ambient(&self) -> bool {
        false
    }
}

#[derive(Clone, Copy)]
pub struct AmbientLight {
    intensity: f64,
}

impl AmbientLight {
    pub const fn new(intensity: f64) -> Self {
        Self { intensity }
    }
}

impl Light for AmbientLight {
    fn intensity(&self) -> f64 {
        self.intensity
    }

    fn get_direction(&self, _point: Vec3f) -> Vec3f {
        Vec3f::new_with_data([0.0, 0.0, 0.0])
    }

    fn get_distance(&self, _point: Vec3f) -> f64 {
        0.0
    }

    fn is_ambient(&self) -> bool {
        true
    }
}

#[derive(Clone, Copy)]
pub struct PointLight {
    intensity: f64,
    position: Vec3f,
}

impl PointLight {
    pub const fn new(intensity: f64, position: Vec3f) -> Self {
        Self {
            intensity,
            position,
        }
    }
}

impl Light for PointLight {
    fn intensity(&self) -> f64 {
        self.intensity
    }

    fn get_direction(&self, point: Vec3f) -> Vec3f {
        (self.position - point).normalize(None)
    }

    fn get_distance(&self, point: Vec3f) -> f64 {
        (self.position - point).length()
    }
}

#[derive(Clone, Copy)]
pub struct DirectionalLight {
    intensity: f64,
    direction: Vec3f,
}

impl DirectionalLight {
    pub const fn new(intensity: f64, direction: Vec3f) -> Self {
        Self {
            intensity,
            direction,
        }
    }
}

impl Light for DirectionalLight {
    fn intensity(&self) -> f64 {
        self.intensity
    }

    fn get_direction(&self, _point: Vec3f) -> Vec3f {
        self.direction
    }

    fn get_distance(&self, _point: Vec3f) -> f64 {
        f64::INFINITY
    }
}

#[derive(Clone, Copy)]
pub enum LightType {
    Point(PointLight),
    Directional(DirectionalLight),
    Ambient(AmbientLight),
}

impl Light for LightType {
    fn intensity(&self) -> f64 {
        match self {
            Self::Ambient(light) => light.intensity(),
            Self::Directional(light) => light.intensity(),
            Self::Point(light) => light.intensity(),
        }
    }
    fn get_direction(&self, point: Vec3f) -> Vec3f {
        match self {
            Self::Ambient(light) => light.get_direction(point),
            Self::Point(light) => light.get_direction(point),
            Self::Directional(light) => light.get_direction(point),
        }
    }

    fn get_distance(&self, point: Vec3f) -> f64 {
        match self {
            Self::Ambient(light) => light.get_distance(point),
            Self::Point(light) => light.get_distance(point),
            Self::Directional(light) => light.get_distance(point),
        }
    }

    fn is_ambient(&self) -> bool {
        matches!(self, Self::Ambient(_))
    }
}
