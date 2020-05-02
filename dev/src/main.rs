use rand::distributions::Uniform;
use rand::rngs::ThreadRng;
use rand::Rng;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point as Sdl2Point;
use sdl2::render::{BlendMode, Canvas};
use sdl2::video::Window;
use sdl2::{EventPump, Sdl, VideoSubsystem};
use std::time::{Duration, Instant};

const WINDOW_WIDTH: u32 = 768;
const WINDOW_HEIGHT: u32 = 768;

const LIGHT_GRAY: Color = Color::RGB(245, 245, 245);
const DARK_GRAY: Color = Color::RGB(40, 40, 40);

#[allow(clippy::cast_precision_loss)]
const POINT_RNG_UPPER: f32 = WINDOW_WIDTH as f32;
const POINT_RNG_LOWER: f32 = 0.0;
const POINT_SPEED_INIT: f32 = 0.0;
const SPEED_INCREMENT: f32 = 0.005;

const FPS: u32 = 60;
const NANOS_PER_FRAME: u128 = (1_000_000_000 / FPS) as u128;
const RELOAD_FRAME_INTERVAL: u32 = FPS * 8;

const SIZE: usize = 32;

#[derive(Clone, Copy)]
struct Point {
    x: f32,
    y: f32,
}

#[derive(Clone, Copy)]
struct Orbiter {
    pos: Point,
    speed: Point,
}

#[allow(clippy::comparison_chain)]
unsafe fn update(orbiters: &mut [Orbiter]) {
    for i in 0..SIZE {
        for j in i..SIZE {
            let a: *mut Orbiter = &mut orbiters[i] as *mut Orbiter;
            let b: *mut Orbiter = &mut orbiters[j] as *mut Orbiter;
            if (*a).pos.x < (*b).pos.x {
                (*a).speed.x += SPEED_INCREMENT;
                (*b).speed.x -= SPEED_INCREMENT;
            } else if (*b).pos.x < (*a).pos.x {
                (*a).speed.x -= SPEED_INCREMENT;
                (*b).speed.x += SPEED_INCREMENT;
            }
            if (*a).pos.y < (*b).pos.y {
                (*a).speed.y += SPEED_INCREMENT;
                (*b).speed.y -= SPEED_INCREMENT;
            } else if (*b).pos.y < (*a).pos.y {
                (*a).speed.y -= SPEED_INCREMENT;
                (*b).speed.y += SPEED_INCREMENT;
            }
        }
    }
    for o in orbiters {
        o.pos.x += o.speed.x;
        o.pos.y += o.speed.y;
    }
}

#[allow(clippy::cast_possible_truncation)]
fn main() {
    let context: Sdl = sdl2::init().unwrap();
    let video: VideoSubsystem = context.video().unwrap();
    let window: Window = video
        .window("rsdl", WINDOW_WIDTH, WINDOW_HEIGHT)
        .opengl()
        .build()
        .unwrap();
    let mut canvas: Canvas<Window> = window.into_canvas().build().unwrap();
    canvas.set_blend_mode(BlendMode::Blend);
    let mut rng: ThreadRng = rand::thread_rng();
    let uniform: Uniform<f32> =
        Uniform::new_inclusive(POINT_RNG_LOWER, POINT_RNG_UPPER);
    let mut orbiters: [Orbiter; SIZE] = [Orbiter {
        pos: Point { x: 0.0, y: 0.0 },
        speed: Point { x: 0.0, y: 0.0 },
    }; SIZE];
    let mut counter: u32 = RELOAD_FRAME_INTERVAL + 1;
    let mut event_pump: EventPump = context.event_pump().unwrap();
    let mut clock: Instant = Instant::now();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        if RELOAD_FRAME_INTERVAL < counter {
            for o in &mut orbiters {
                o.pos.x = rng.sample(uniform);
                o.pos.y = rng.sample(uniform);
                o.speed.x = POINT_SPEED_INIT;
                o.speed.y = POINT_SPEED_INIT;
            }
            counter = 0;
        } else {
            unsafe {
                update(&mut orbiters);
            }
            counter += 1;
        }
        canvas.set_draw_color(DARK_GRAY);
        canvas.clear();
        canvas.set_draw_color(LIGHT_GRAY);
        for o in &orbiters {
            canvas
                .draw_line(
                    Sdl2Point::new(o.pos.x as i32, o.pos.y as i32),
                    Sdl2Point::new(
                        (o.pos.x + (o.speed.x * 4.0)) as i32,
                        (o.pos.y + (o.speed.y * 4.0)) as i32,
                    ),
                )
                .unwrap();
        }
        canvas.present();
        std::thread::sleep(Duration::from_nanos(
            (NANOS_PER_FRAME - clock.elapsed().as_nanos()) as u64,
        ));
        clock = Instant::now();
    }
}
