use sdl2::event::Event;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::{BlendMode, Canvas};
use sdl2::video::Window;
use sdl2::{EventPump, Sdl, VideoSubsystem};
use std::io;
use std::io::Write;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const PCG_CONSTANT: u64 = 0x853c_49e6_748f_ea9b;

const WINDOW_WIDTH: u32 = 768;
const WINDOW_HEIGHT: u32 = 768;

const LIGHT_GRAY: Color = Color::RGB(245, 245, 245);
const DARK_GRAY: Color = Color::RGB(40, 40, 40);
const TEAL: Color = Color::RGBA(45, 210, 195, 90);

const LINE_WIDTH: u8 = 3;
const RECT_PAD: i16 = 12;

const SPEED_INIT: f32 = 0.0;
const SPEED_INCREMENT: f32 = 0.0065;
const TRAIL: f32 = 10.0;

const FPS: u32 = 60;
const NANOS_PER_FRAME: u64 = (1_000_000_000 / FPS) as u64;
const RELOAD_FRAME_INTERVAL: u32 = FPS * 10;

const SIZE: usize = 32;
const SIZE_MINUS_1: usize = SIZE - 1;

struct PcgRng {
    state: u64,
    increment: u64,
}

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

struct State {
    frame_clock: Instant,
    event_pump: EventPump,
    reset_counter: u32,
    benchmark_clock: Instant,
    benchmark_elapsed: f32,
    benchmark_counter: f32,
}

fn negate_u32(x: u32) -> u32 {
    (-(x as i32)) as u32
}

fn pcg_32(rng: &mut PcgRng) -> u32 {
    let state: u64 = rng.state;
    rng.state = (state * 6_364_136_223_846_793_005) + (rng.increment | 1);
    let xor_shift: u32 = (((state >> 18) ^ state) >> 27) as u32;
    let rotate: u32 = (state >> 59) as u32;
    (xor_shift >> rotate) | (xor_shift << (negate_u32(rotate) & 31))
}

fn pcg_32_bound(rng: &mut PcgRng, bound: u32) -> u32 {
    let threshold: u32 = negate_u32(bound) % bound;
    loop {
        let value: u32 = pcg_32(rng);
        if threshold <= value {
            return value % bound;
        }
    }
}

fn get_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

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

fn render(canvas: &mut Canvas<Window>, orbiters: &[Orbiter]) {
    for o in orbiters.iter().take(SIZE_MINUS_1) {
        canvas
            .thick_line(
                o.pos.x as i16,
                o.pos.y as i16,
                (o.pos.x - (o.speed.x * TRAIL)) as i16,
                (o.pos.y - (o.speed.y * TRAIL)) as i16,
                LINE_WIDTH,
                LIGHT_GRAY,
            )
            .unwrap();
    }
    let o: &Orbiter = &orbiters[SIZE_MINUS_1];
    let x: i16 = o.pos.x as i16;
    let y: i16 = o.pos.y as i16;
    let speed_x: i16 = x - ((o.speed.x * TRAIL) as i16);
    let speed_y: i16 = y - ((o.speed.y * TRAIL) as i16);
    let (a_x, b_x): (i16, i16) = {
        if x < speed_x {
            (x - RECT_PAD, speed_x + RECT_PAD)
        } else {
            (speed_x - RECT_PAD, x + RECT_PAD)
        }
    };
    let (a_y, b_y): (i16, i16) = {
        if y < speed_y {
            (y - RECT_PAD, speed_y + RECT_PAD)
        } else {
            (speed_y - RECT_PAD, y + RECT_PAD)
        }
    };
    canvas.box_(a_x, a_y, b_x, b_y, TEAL).unwrap();
    canvas
        .thick_line(x, y, speed_x, speed_y, LINE_WIDTH, LIGHT_GRAY)
        .unwrap();
}

fn benchmark(state: &mut State) {
    state.benchmark_elapsed += state.benchmark_clock.elapsed().as_secs_f32();
    state.benchmark_clock = Instant::now();
    state.benchmark_counter += 1.0;
    if 1.0 < state.benchmark_elapsed {
        print!(
            "{:>8.2} fps\r",
            state.benchmark_counter / state.benchmark_elapsed,
        );
        io::stdout().flush().unwrap();
        state.benchmark_counter = 0.0;
        state.benchmark_elapsed = 0.0;
    }
}

fn sleep(state: &mut State) {
    let frame_elapsed: u64 = state.frame_clock.elapsed().as_nanos() as u64;
    if frame_elapsed < NANOS_PER_FRAME {
        std::thread::sleep(Duration::from_nanos(
            NANOS_PER_FRAME - frame_elapsed,
        ));
    }
    state.frame_clock = Instant::now();
}

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
    let mut rng: PcgRng = PcgRng {
        state: PCG_CONSTANT * get_seconds(),
        increment: PCG_CONSTANT * get_seconds(),
    };
    let mut orbiters: [Orbiter; SIZE] = [Orbiter {
        pos: Point { x: 0.0, y: 0.0 },
        speed: Point { x: 0.0, y: 0.0 },
    }; SIZE];
    let mut state: State = State {
        frame_clock: Instant::now(),
        event_pump: context.event_pump().unwrap(),
        reset_counter: RELOAD_FRAME_INTERVAL,
        benchmark_clock: Instant::now(),
        benchmark_elapsed: 0.0,
        benchmark_counter: 0.0,
    };
    loop {
        for event in state.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return,
                _ => {}
            }
        }
        if RELOAD_FRAME_INTERVAL < state.reset_counter {
            for o in &mut orbiters {
                o.pos.x = pcg_32_bound(&mut rng, WINDOW_WIDTH) as f32;
                o.pos.y = pcg_32_bound(&mut rng, WINDOW_HEIGHT) as f32;
                o.speed.x = SPEED_INIT;
                o.speed.y = SPEED_INIT;
            }
            state.reset_counter = 0;
        } else {
            unsafe {
                update(&mut orbiters);
            }
            state.reset_counter += 1;
        }
        canvas.set_draw_color(DARK_GRAY);
        canvas.clear();
        render(&mut canvas, &orbiters);
        canvas.present();
        benchmark(&mut state);
        sleep(&mut state);
    }
}
