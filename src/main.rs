use glam::UVec2;
use raylib::prelude::*;
use raylib::{ffi::SetTraceLogLevel, prelude::TraceLogLevel};

mod sketch;

const TIMESTEP: f32 = 1.0 / sketch::FRAMES_PER_SECOND as f32;

fn main() {
    // --- Raylib and Window Initialization ---
    let (mut rl, mut rlt) = raylib::init().title("raylib-rs-native-template").build();
    unsafe {
        SetTraceLogLevel(TraceLogLevel::LOG_WARNING as i32);
    }
    let window_dims = UVec2::new(1280, 720);
    rl.set_window_size(window_dims.x as i32, window_dims.y as i32);
    center_window(&mut rl, window_dims.x as i32, window_dims.y as i32);

    // --- State Initialization ---
    // We must initialize the state *after* raylib so we can create the render texture.
    let mut state = sketch::State::new(&mut rl, &rlt);

    // --- Main Game Loop ---
    while state.running && !rl.window_should_close() {
        // Process inputs from the user
        sketch::process_events_and_input(&mut rl, &mut state);

        // --- Fixed Timestep Update Logic ---
        let dt = rl.get_frame_time();
        state.time_since_last_update += dt;
        while state.time_since_last_update > TIMESTEP {
            state.time_since_last_update -= TIMESTEP;
            sketch::step(&mut rl, &mut rlt, &mut state);
        }

        // --- Drawing Logic ---
        let mut draw_handle = rl.begin_drawing(&rlt);

        // The main draw function now also needs the thread handle to manage texture modes.
        sketch::draw(&mut state, &mut draw_handle, &rlt);
    }
}

/// Centers the window on the current monitor.
pub fn center_window(rl: &mut RaylibHandle, width: i32, height: i32) {
    let monitor = get_current_monitor();
    let monitor_width = get_monitor_width(monitor);
    let monitor_height = get_monitor_height(monitor);
    let monitor_pos = get_monitor_position(monitor);

    if let Ok(name) = get_monitor_name(monitor) {
        println!(
            "Centering on Monitor {}: '{}' ({}x{}) at ({}, {})",
            monitor, name, monitor_width, monitor_height, monitor_pos.x, monitor_pos.y
        );
    }
    let x = monitor_pos.x as i32 + (monitor_width - width) / 2;
    let y = monitor_pos.y as i32 + (monitor_height - height) / 2;
    rl.set_window_position(x, y);
}
