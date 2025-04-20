#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::todo)]
#![warn(clippy::dbg_macro)]

use std::mem::swap;
use std::rc::Rc;
use std::sync::Arc;
use std::{error::Error, f64};

use pixels::{Pixels, SurfaceTexture};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{KeyEvent, StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowAttributes, WindowId},
};

mod geometry;
mod lights;
mod materials;
mod shapes;

use geometry::{Vec3f, Vec4f};
use lights::{AmbientLight, DirectionalLight, Light, LightType, PointLight};
use materials::{
    BLUE_MATERIAL, GLASS_MATERIAL, GREEN_MATERIAL, MIRROR_MATERIAL, Material, RED_MATERIAL,
};
use shapes::{InfinityPlane, Shape, Sphere};

const PI: f64 = f64::consts::PI;
const MAX_DEPTH: u32 = 4;
const EPSILON: f64 = 1e-3;
const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;

const BACKGROUND_COLOR: Vec3f = Vec3f::const_new_with_data([0.2, 0.7, 0.8]);

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
    shapes: &Rc<Vec<Box<dyn Shape>>>,
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
    shapes: &Rc<Vec<Box<dyn Shape>>>,
) -> Option<(Vec3f, Vec3f, Material)> {
    let mut closest_distance = f64::INFINITY;
    let mut hit_point = None;

    for shape in shapes.iter() {
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
    shapes: &Rc<Vec<Box<dyn Shape>>>,
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

struct Raytracer<'win> {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'win>>,
}

impl Raytracer<'_> {
    const fn new() -> Self {
        Self {
            window: None,
            pixels: None,
        }
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn to_u8(color: f64) -> u8 {
    (color.clamp(0.0, 1.0) * 255.0).round() as u8
}

impl ApplicationHandler for Raytracer<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("App resumed!");

        let window = match event_loop.create_window(
            WindowAttributes::default()
                .with_title("Raytracer")
                .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT)),
        ) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("Failed to create window: {e}");
                return; // или handle иначе
            }
        };

        let window_arc = Arc::new(window);
        self.window = Some(Arc::clone(&window_arc));

        let surface_texture = SurfaceTexture::new(WIDTH, HEIGHT, Arc::clone(&window_arc));
        match Pixels::new(WIDTH, HEIGHT, surface_texture) {
            Ok(p) => self.pixels = Some(p),
            Err(e) => {
                eprintln!("Failed to create Pixels: {e}");
            }
        };
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        #[allow(unused_variables)] window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Close requested");
                event_loop.exit();
            }

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        state: winit::event::ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                println!("Escape pressed");
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                if let Some(pixels) = &mut self.pixels {
                    let frame = pixels.frame_mut();
                    let fov: f64 = PI / 3.0;

                    let shapes: Rc<Vec<Box<dyn Shape>>> = Rc::new(vec![
                        Box::new(Sphere::new(
                            Vec3f::new_with_data([0.0, -1.0, -7.0]),
                            2.0,
                            RED_MATERIAL,
                        )),
                        Box::new(Sphere::new(
                            Vec3f::new_with_data([2.0, 0.0, -4.0]),
                            1.0,
                            GREEN_MATERIAL,
                        )),
                        Box::new(Sphere::new(
                            Vec3f::new_with_data([-2.0, 1.0, -5.0]),
                            1.5,
                            BLUE_MATERIAL,
                        )),
                        Box::new(Sphere::new(
                            Vec3f::new_with_data([-0.5, -0.75, -2.0]),
                            0.25,
                            GLASS_MATERIAL,
                        )),
                        Box::new(Sphere::new(
                            Vec3f::new_with_data([0.5, 1.5, -3.5]),
                            0.4,
                            MIRROR_MATERIAL,
                        )),
                        Box::new(InfinityPlane::new(
                            Vec3f::new_with_data([0.0, -2.9, 0.0]),
                            Vec3f::new_with_data([0.0, -10.0, -1.0]),
                            MIRROR_MATERIAL,
                        )),
                    ]);

                    let lights = vec![
                        LightType::Ambient(AmbientLight::new(0.1)),
                        LightType::Directional(DirectionalLight::new(
                            2.0,
                            Vec3f::new_with_data([-1.0, -1.0, -1.0]).normalize(None),
                        )),
                        LightType::Point(PointLight::new(
                            2.0,
                            Vec3f::new_with_data([2.0, 5.0, 0.0]),
                        )),
                        LightType::Point(PointLight::new(
                            0.5,
                            Vec3f::new_with_data([-1.0, -1.0, 5.0]),
                        )),
                    ];

                    for j in 0..HEIGHT {
                        for i in 0..WIDTH {
                            let x = (2.0 * (f64::from(i) + 0.5) / f64::from(WIDTH) - 1.0)
                                * (fov / 2.0).tan()
                                * f64::from(WIDTH)
                                / f64::from(HEIGHT);
                            let y = -(2.0 * (f64::from(j) + 0.5) / f64::from(HEIGHT) - 1.0)
                                * (fov / 2.0).tan();
                            let dir = Vec3f::new_with_data([x, y, -1.0]).normalize(None);
                            let color = cast_ray(
                                Vec3f::new_with_data([0.0, 0.0, 2.0]),
                                dir,
                                &shapes.clone(),
                                &lights.clone(),
                                0,
                            );

                            let index = ((j * WIDTH + i) * 4) as usize;
                            frame[index] = to_u8(color[0]);
                            frame[index + 1] = to_u8(color[1]);
                            frame[index + 2] = to_u8(color[2]);
                            frame[index + 3] = 255;
                        }
                    }
                    match pixels.render() {
                        Ok(()) => (),
                        Err(err) => eprint!("Error with render pixels: {err}"),
                    };
                }
            }

            _ => {}
        }
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        if cause == StartCause::Init {
            println!("Starting app!");
        }

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new()?;
    let mut app = Raytracer::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}
