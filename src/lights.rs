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
    pub fn new(intensity: f64) -> Self {
        AmbientLight { intensity }
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
    pub fn new(intensity: f64, position: Vec3f) -> Self {
        PointLight {
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
    pub fn new(intensity: f64, direction: Vec3f) -> Self {
        DirectionalLight {
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
            LightType::Ambient(light) => light.intensity(),
            LightType::Directional(light) => light.intensity(),
            LightType::Point(light) => light.intensity(),
        }
    }
    fn get_direction(&self, _point: Vec3f) -> Vec3f {
        match self {
            LightType::Ambient(light) => light.get_direction(_point),
            LightType::Point(light) => light.get_direction(_point),
            LightType::Directional(light) => light.get_direction(_point),
        }
    }

    fn get_distance(&self, _point: Vec3f) -> f64 {
        match self {
            LightType::Ambient(light) => light.get_distance(_point),
            LightType::Point(light) => light.get_distance(_point),
            LightType::Directional(light) => light.get_distance(_point),
        }
    }

    fn is_ambient(&self) -> bool {
        matches!(self, LightType::Ambient(_))
    }
}
