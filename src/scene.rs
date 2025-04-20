use std::mem::swap;

use crate::Vec3f;
use crate::{BACKGROUND_COLOR, EPSILON, MAX_DEPTH};
use crate::{
    lights::{Light, LightType},
    materials::Material,
    shapes::{Intersectable, Shape, ShapeType},
};

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn to_u8(color: f64) -> u8 {
    (color.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn reflect(direction: Vec3f, normal: Vec3f) -> Vec3f {
    direction - normal * (direction * normal) * 2.0
}

fn refract(direction: Vec3f, normal: Vec3f, refractive_index: f64) -> Vec3f {
    let mut cosi = (direction * normal).clamp(-1.0, 1.0);
    let mut ior_in = 1.0;
    let mut ior_out = refractive_index;
    let mut n = normal;

    if cosi < 0.0 {
        cosi *= -1.0;
        swap(&mut ior_in, &mut ior_out);
        n = -n;
    }

    let eta = ior_in / ior_out;
    let k = (eta * eta).mul_add(-cosi.mul_add(-cosi, 1.0), 1.0);

    if k < 0.0 {
        return Vec3f::new_with_data([0.0, 0.0, 0.0]);
    }
    direction * eta + n * eta.mul_add(cosi, -k.sqrt())
}

fn adjust_ray_origin(direction: Vec3f, point: Vec3f, normal: Vec3f) -> Vec3f {
    if direction * normal < 0.0 {
        return point - normal * EPSILON;
    }

    point + normal * EPSILON
}

fn is_in_shadow(
    normal: Vec3f,
    point: Vec3f,
    light_direction: Vec3f,
    light_distance: f64,
    shapes: &[ShapeType],
) -> (bool, Option<(Vec3f, Vec3f)>) {
    let shadow_origin = adjust_ray_origin(light_direction, point, normal);
    let scene_intersect_option = scene_intersect(shadow_origin, light_direction, shapes);
    let Some(scene_intersect_result) = scene_intersect_option else {
        return (false, None);
    };

    let (shadow_hit, _, _) = scene_intersect_result;

    (
        (shadow_hit - shadow_origin).length() < light_distance,
        Some((shadow_origin, shadow_hit)),
    )
}

fn scene_intersect(
    origin: Vec3f,
    direction: Vec3f,
    shapes: &[ShapeType],
) -> Option<(Vec3f, Vec3f, Material)> {
    let mut closest_distance = f64::INFINITY;
    let mut hit_point = None;

    for shape in shapes {
        let dist_i = shape.ray_intersect(origin, direction);

        if let Some(distance) = dist_i {
            if distance < closest_distance {
                let hit = origin + direction * distance;
                let normal = shape.get_normal(hit);
                let material = shape.get_material();
                closest_distance = distance;

                hit_point = Some((hit, normal, material));
            }
        }
    }

    if closest_distance > 1000.0 {
        return None;
    }

    hit_point
}

fn cast_ray(
    origin: Vec3f,
    direction: Vec3f,
    shapes: &[ShapeType],
    lights: &[LightType],
    depth: u32,
) -> Vec3f {
    let scene_intersect_option = scene_intersect(origin, direction, shapes);

    let Some(scene_intersect_result) = scene_intersect_option else {
        return BACKGROUND_COLOR;
    };

    if depth > MAX_DEPTH {
        return BACKGROUND_COLOR;
    }

    let (hit, normal, material) = scene_intersect_result;

    let reflect_direction = reflect(direction, normal).normalize(None);
    let reflect_origin = adjust_ray_origin(reflect_direction, hit, normal);
    let reflect_color = cast_ray(reflect_origin, reflect_direction, shapes, lights, depth + 1);

    let refract_direction = refract(direction, normal, material.refractive_index()).normalize(None);
    let refract_origin = adjust_ray_origin(refract_direction, hit, normal);
    let refract_color = cast_ray(refract_origin, refract_direction, shapes, lights, depth + 1);

    let mut diffuse_light_intensity = 0.0;
    let mut specular_light_intensity = 0.0;
    let mut ambient_light_intensity = 0.0;
    for light in lights {
        if light.is_ambient() {
            ambient_light_intensity += light.intensity();
            continue;
        }

        let light_direction = light.get_direction(hit);
        let light_distance = light.get_distance(hit);
        let reflect = reflect(light_direction, normal) * direction;

        let (is_in_shadow, shadow_point) =
            is_in_shadow(normal, hit, light_direction, light_distance, shapes);

        if is_in_shadow {
            if let Some((origin, hit)) = shadow_point {
                if (hit - origin).length() < light_distance {
                    continue;
                }
            }
        }

        diffuse_light_intensity += light.intensity() * f64::max(0.0, light_direction * normal);
        specular_light_intensity +=
            (reflect.max(0.0)).powf(material.specular_exponent()) * light.intensity();
    }

    calculate_final_color(
        material,
        ambient_light_intensity,
        diffuse_light_intensity,
        specular_light_intensity,
        reflect_color,
        refract_color,
    )
}

fn calculate_final_color(
    material: Material,
    ambient_light_intensity: f64,
    diffuse_light_intensity: f64,
    specular_light_intensity: f64,
    reflect_color: Vec3f,
    refract_color: Vec3f,
) -> Vec3f {
    let albedo = material.albedo();
    material.ambient_color() * ambient_light_intensity
        + material.diffuse_color() * diffuse_light_intensity * albedo[0]
        + Vec3f::new_with_data([1.0, 1.0, 1.0]) * specular_light_intensity * albedo[1]
        + reflect_color * albedo[2]
        + refract_color * albedo[3]
}

pub struct Scene {
    shapes: Vec<ShapeType>,
    lights: Vec<LightType>,
}

impl Scene {
    pub const fn new(shapes: Vec<ShapeType>, lights: Vec<LightType>) -> Self {
        Self { shapes, lights }
    }

    pub fn push_light(&mut self, light: LightType) {
        self.lights.push(light);
    }

    pub fn push_shape(&mut self, shape: ShapeType) {
        self.shapes.push(shape);
    }

    pub fn render_scene(&self, frame: &mut [u8], height: u32, width: u32, fov: f64) {
        for j in 0..height {
            for i in 0..width {
                let x = (2.0 * (f64::from(i) + 0.5) / f64::from(width) - 1.0)
                    * (fov / 2.0).tan()
                    * f64::from(width)
                    / f64::from(height);
                let y = -(2.0 * (f64::from(j) + 0.5) / f64::from(height) - 1.0) * (fov / 2.0).tan();
                let dir = Vec3f::new_with_data([x, y, -1.0]).normalize(None);
                let color = cast_ray(
                    Vec3f::new_with_data([0.0, 0.0, 2.0]),
                    dir,
                    &self.shapes,
                    &self.lights,
                    0,
                );

                let index = ((j * width + i) * 4) as usize;
                frame[index] = to_u8(color[0]);
                frame[index + 1] = to_u8(color[1]);
                frame[index + 2] = to_u8(color[2]);
                frame[index + 3] = 255;
            }
        }
    }
}
