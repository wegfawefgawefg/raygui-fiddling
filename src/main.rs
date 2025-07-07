use glam::UVec2;
use raylib::prelude::*;
use raylib::{ffi::SetTraceLogLevel, prelude::TraceLogLevel};

mod sketch;

const TIMESTEP: f32 = 1.0 / sketch::FRAMES_PER_SECOND as f32;

fn main() {
    // Initialize the game state and Raylib
    let mut state = sketch::State::new();
    let (mut rl, mut rlt) = raylib::init().title("raylib-rs-native-template").build();
    unsafe {
        SetTraceLogLevel(TraceLogLevel::LOG_WARNING as i32);
    }

    // --- Native Resolution Setup ---
    // We only need one set of dimensions now.
    let window_dims = UVec2::new(1280, 720);
    let fullscreen = false;
    rl.set_window_size(window_dims.x as i32, window_dims.y as i32);
    if fullscreen {
        rl.toggle_fullscreen();
        // Adjust window size to the full screen dimensions if toggled
        rl.set_window_size(rl.get_screen_width(), rl.get_screen_height());
    }

    // Center the window on the primary monitor.
    // We call this *after* setting the size and fullscreen mode.
    center_window(&mut rl, window_dims.x as i32, window_dims.y as i32);

    // --- Main Game Loop ---
    while state.running && !rl.window_should_close() {
        // Process inputs from the user
        sketch::process_events_and_input(&mut rl, &mut state);

        // --- Fixed Timestep Update Logic ---
        // This ensures the game logic runs at a consistent rate.
        let dt = rl.get_frame_time();
        state.time_since_last_update += dt;
        while state.time_since_last_update > TIMESTEP {
            state.time_since_last_update -= TIMESTEP;
            sketch::step(&mut rl, &mut rlt, &mut state);
        }

        // --- Drawing Logic ---
        // All drawing happens directly to the screen buffer now.
        let mut draw_handle = rl.begin_drawing(&rlt);
        draw_handle.clear_background(Color::BLACK);

        // The sketch::draw function now receives the main draw handle.
        // You may need to update its function signature in sketch.rs to accept &mut RaylibDrawHandle.
        sketch::draw(&mut state, &mut draw_handle);
    }
}

/// Centers the window on the current monitor.
pub fn center_window(rl: &mut RaylibHandle, width: i32, height: i32) {
    let monitor = get_current_monitor();
    let monitor_width = get_monitor_width(monitor);
    let monitor_height = get_monitor_height(monitor);
    let monitor_pos = get_monitor_position(monitor);

    // For debugging purposes, print which monitor is being used.
    if let Ok(name) = get_monitor_name(monitor) {
        println!(
            "Centering on Monitor {}: '{}' ({}x{}) at ({}, {})",
            monitor, name, monitor_width, monitor_height, monitor_pos.x, monitor_pos.y
        );
    }

    // Calculate the centered position
    let x = monitor_pos.x as i32 + (monitor_width - width) / 2;
    let y = monitor_pos.y as i32 + (monitor_height - height) / 2;

    // Set the new window position
    rl.set_window_position(x, y);
}
