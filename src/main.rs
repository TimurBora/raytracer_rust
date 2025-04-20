#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
//#![warn(clippy::cargo)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::todo)]
#![warn(clippy::dbg_macro)]

use std::sync::Arc;
use std::{error::Error, f64};

use lights::init_default_lights;
use pixels::{Pixels, SurfaceTexture};
use scene::Scene;
use shapes::init_default_shapes;
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
mod scene;
mod shapes;

use geometry::{Vec3f, Vec4f};
use materials::{
    BLUE_MATERIAL, GLASS_MATERIAL, GREEN_MATERIAL, MIRROR_MATERIAL, Material, RED_MATERIAL,
};

const PI: f64 = f64::consts::PI;
const MAX_DEPTH: u32 = 4;
const EPSILON: f64 = 1e-3;
const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const FOV: f64 = PI / 3.0;

const BACKGROUND_COLOR: Vec3f = Vec3f::const_new_with_data([0.2, 0.7, 0.8]);

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
                return;
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

                    let scene = Scene::new(init_default_shapes(), init_default_lights());
                    scene.render_scene(frame, HEIGHT, WIDTH, FOV);

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
