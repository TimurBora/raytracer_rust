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
use lights::*;
use materials::*;
use shapes::*;

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
    let mut etai = 1.0;
    let mut etat = refractive_index;
    let mut n = normal;

    if cosi < 0.0 {
        cosi *= -1.0;
        swap(&mut etai, &mut etat);
        n = -n;
    }

    let eta = etai / etat;
    let k = 1.0 - eta * eta * (1.0 - cosi * cosi);

    if k < 0.0 {
        return Vec3f::new_with_data([0.0, 0.0, 0.0]);
    }
    direction * eta + n * (eta * cosi - k.sqrt())
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
    shapes: Rc<Vec<Box<dyn Shape>>>,
) -> (bool, Option<(Vec3f, Vec3f)>) {
    let shadow_origin = adjust_ray_origin(light_direction, point, normal);
    let scene_intersect_option =
        scene_intersect(shadow_origin, light_direction, Rc::clone(&shapes));
    let scene_intersect_result = match scene_intersect_option {
        Some(hit_point) => hit_point,
        None => return (false, None),
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
    shapes: Rc<Vec<Box<dyn Shape>>>,
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
    shapes: Rc<Vec<Box<dyn Shape>>>,
    lights: Vec<LightType>,
    depth: u32,
) -> Vec3f {
    let scene_intersect_option = scene_intersect(origin, direction, Rc::clone(&shapes));
    let scene_intersect_result = match scene_intersect_option {
        Some(hit_point) => hit_point,
        None => return BACKGROUND_COLOR,
    };

    if depth > MAX_DEPTH {
        return BACKGROUND_COLOR;
    }

    let (hit, normal, material) = scene_intersect_result;

    let reflect_direction = reflect(direction, normal).normalize(None);
    let reflect_origin = adjust_ray_origin(reflect_direction, hit, normal);
    let reflect_color = cast_ray(
        reflect_origin,
        reflect_direction,
        Rc::clone(&shapes),
        lights.clone(),
        depth + 1,
    );

    let refract_direction = refract(direction, normal, material.refractive_index()).normalize(None);
    let refract_origin = adjust_ray_origin(refract_direction, hit, normal);
    let refract_color = cast_ray(
        refract_origin,
        refract_direction,
        Rc::clone(&shapes),
        lights.clone(),
        depth + 1,
    );

    let mut diffuse_light_intensity = 0.0;
    let mut specular_light_intensity = 0.0;
    let mut ambient_light_intensity = 0.0;
    for light in lights.iter() {
        if light.is_ambient() {
            ambient_light_intensity += light.intensity();
            continue;
        }

        let light_direction = light.get_direction(hit);
        let light_distance = light.get_distance(hit);
        let reflect = reflect(light_direction, normal) * direction;

        let (is_in_shadow, shadow_point) = is_in_shadow(
            normal,
            hit,
            light_direction,
            light_distance,
            Rc::clone(&shapes),
        );

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
    fn new() -> Self {
        Self {
            window: None,
            pixels: None,
        }
    }
}

impl ApplicationHandler for Raytracer<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("App resumed!");

        let window = event_loop
            .create_window(
                WindowAttributes::default()
                    .with_title("Raytracer")
                    .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT)),
            )
            .expect("Failed to create window");

        let window_arc = Arc::new(window);

        self.window = Some(Arc::clone(&window_arc));

        let surface_texture = SurfaceTexture::new(WIDTH, HEIGHT, Arc::clone(&window_arc));
        self.pixels = Some(Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap());
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
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
                            let x = (2.0 * (i as f64 + 0.5) / WIDTH as f64 - 1.0)
                                * (fov / 2.0).tan()
                                * WIDTH as f64
                                / HEIGHT as f64;
                            let y =
                                -(2.0 * (j as f64 + 0.5) / HEIGHT as f64 - 1.0) * (fov / 2.0).tan();
                            let dir = Vec3f::new_with_data([x, y, -1.0]).normalize(None);
                            let color = cast_ray(
                                Vec3f::new_with_data([0.0, 0.0, 2.0]),
                                dir,
                                shapes.clone(),
                                lights.clone(),
                                0,
                            );

                            let index = ((j * WIDTH + i) * 4) as usize;
                            frame[index] = (color[0] * 255.0) as u8;
                            frame[index + 1] = (color[1] * 255.0) as u8;
                            frame[index + 2] = (color[2] * 255.0) as u8;
                            frame[index + 3] = 255;
                        }
                    }
                    pixels.render().unwrap();
                }
            }

            _ => {}
        }
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        if let StartCause::Init = cause {
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
