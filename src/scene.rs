use core::f64;
use rayon::prelude::*;
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
    (color * 255.0).round() as u8
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
    shapes
        .iter()
        .filter_map(|shape| {
            shape.ray_intersect(origin, direction).map(|distance| {
                let hit = origin + direction * distance;
                let normal = shape.get_normal(hit);
                let material = shape.get_material();
                (distance, (hit, normal, material))
            })
        })
        .min_by(
            |a, b| match (a.0.partial_cmp(&b.0), a.0.is_nan(), b.0.is_nan()) {
                (Some(order), false, false) => order,
                (_, true, false) => std::cmp::Ordering::Greater,
                (_, false, true) => std::cmp::Ordering::Less,
                _ => std::cmp::Ordering::Equal,
            },
        )
        .filter(|(dist, _)| *dist < 1000.0)
        .map(|(_, result)| result)
}

fn compute_lighthing(
    hit: Vec3f,
    normal: Vec3f,
    direction: Vec3f,
    lights: &[LightType],
    material: Material,
    shapes: &[ShapeType],
) -> (f64, f64, f64) {
    let (ambient, specular, diffuse) = lights
        .iter()
        .map(|light| {
            if light.is_ambient() {
                return (light.intensity(), 0.0, 0.0);
            }

            let light_direction = light.get_direction(hit);
            let light_distance = light.get_distance(hit);
            let reflect = reflect(light_direction, normal) * direction;

            let (shadowed, shadow_point) =
                is_in_shadow(normal, hit, light_direction, light_distance, shapes);

            if shadowed {
                if let Some((origin, hit)) = shadow_point {
                    if (hit - origin).length() < light_distance {
                        return (0.0, 0.0, 0.0);
                    }
                }
            }

            let diffuse = light.intensity() * f64::max(0.0, light_direction * normal);
            let specular = reflect.max(0.0).powf(material.specular_exponent()) * light.intensity();

            (0.0, specular, diffuse)
        })
        .fold((0.0, 0.0, 0.0), |acc, val| {
            (acc.0 + val.0, acc.1 + val.1, acc.2 + val.2)
        });

    (ambient, diffuse, specular)
}

fn cast_ray(
    origin: Vec3f,
    direction: Vec3f,
    shapes: &[ShapeType],
    lights: &[LightType],
    depth: u32,
) -> Vec3f {
    if depth > MAX_DEPTH {
        return BACKGROUND_COLOR;
    }

    let Some((hit, normal, material)) = scene_intersect(origin, direction, shapes) else {
        return BACKGROUND_COLOR;
    };

    let reflect_direction = reflect(direction, normal).normalize(None);
    let reflect_origin = adjust_ray_origin(reflect_direction, hit, normal);
    let reflect_color = cast_ray(reflect_origin, reflect_direction, shapes, lights, depth + 1);

    let refract_direction = refract(direction, normal, material.refractive_index()).normalize(None);
    let refract_origin = adjust_ray_origin(refract_direction, hit, normal);
    let refract_color = cast_ray(refract_origin, refract_direction, shapes, lights, depth + 1);

    let (ambient, diffuse, specular) =
        compute_lighthing(hit, normal, direction, lights, material, shapes);

    calculate_final_color(
        material,
        ambient,
        diffuse,
        specular,
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

    #[allow(dead_code)]
    pub fn push_light(&mut self, light: LightType) {
        self.lights.push(light);
    }

    #[allow(dead_code)]
    pub fn push_shape(&mut self, shape: ShapeType) {
        self.shapes.push(shape);
    }

    pub fn render_scene(&self, frame: &mut [u8], height: u32, width: u32, fov: f64) {
        let fov_tan = (fov / 2.0).tan();
        let origin = Vec3f::new_with_data([0.0, 0.0, 2.0]);
        frame
            .par_chunks_mut(4)
            .enumerate()
            .for_each(|(index, pixel)| {
                let i_usize = index % (width as usize);
                let j_usize = index / (width as usize);

                let Ok(i) = u32::try_from(i_usize) else {
                    eprintln!("Index i out of u32 range: {i_usize}");
                    return;
                };

                let Ok(j) = u32::try_from(j_usize) else {
                    eprintln!("Index j out of u64 range: {j_usize}");
                    return;
                };

                let x = (2.0 * (f64::from(i) + 0.5) / f64::from(width) - 1.0)
                    * fov_tan
                    * f64::from(width)
                    / f64::from(height);
                let y = -(2.0 * (f64::from(j) + 0.5) / f64::from(height) - 1.0) * fov_tan;
                let dir = Vec3f::new_with_data([x, y, -1.0]).normalize(None);
                let color = cast_ray(origin, dir, &self.shapes, &self.lights, 0);

                pixel[0] = to_u8(color[0]);
                pixel[1] = to_u8(color[1]);
                pixel[2] = to_u8(color[2]);
                pixel[3] = 255;
            });
    }
}
