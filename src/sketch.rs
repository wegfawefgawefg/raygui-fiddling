use glam::Vec2;
use raylib::prelude::*;

pub const FRAMES_PER_SECOND: u32 = 60;

// Default values for our GUI controls
const DEFAULT_ROTATION_SPEED: f32 = 1.0;
const DEFAULT_RECT_COLOR: Color = Color::RED;
const DEFAULT_DRAW_RECTS: bool = true;

pub struct State {
    pub running: bool,
    pub time_since_last_update: f32,

    // --- GUI State ---
    /// Controls the visibility of the GUI window.
    pub show_gui_window: bool,
    /// The value controlled by the slider, used for rotation speed.
    pub rotation_speed: f32,
    /// The value controlled by the checkbox, used to toggle drawing the rectangles.
    pub draw_rects: bool,
    /// The color controlled by the color picker.
    pub rect_color: Color,
}

impl State {
    pub fn new() -> Self {
        Self {
            running: true,
            time_since_last_update: 0.0,
            // Initialize GUI state variables
            show_gui_window: true,
            rotation_speed: DEFAULT_ROTATION_SPEED,
            draw_rects: DEFAULT_DRAW_RECTS,
            rect_color: DEFAULT_RECT_COLOR,
        }
    }
}

pub fn process_events_and_input(rl: &mut RaylibHandle, state: &mut State) {
    if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_ESCAPE) {
        state.running = false;
    }
    // Press 'G' to toggle the GUI window visibility
    if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_G) {
        state.show_gui_window = !state.show_gui_window;
    }
}

pub fn step(_rl: &mut RaylibHandle, _rlt: &mut RaylibThread, _state: &mut State) {
    // Nothing to update in the fixed step for this demo
}

/// Main drawing function, which calls our sketch and GUI drawing functions.
pub fn draw(state: &mut State, d: &mut RaylibDrawHandle) {
    // Draw the original sketch elements
    draw_sketch(state, d);

    // Draw the GUI on top
    draw_gui(state, d);

    // Draw some helper text
    d.draw_text("Press 'G' to toggle GUI", 12, 12, 20, Color::WHITE);
    let mouse_pos = d.get_mouse_position();
    d.draw_circle(mouse_pos.x as i32, mouse_pos.y as i32, 6.0, Color::GREEN);
}

/// Draws the spinning rectangles, now influenced by the GUI state.
fn draw_sketch(state: &State, d: &mut RaylibDrawHandle) {
    // Only draw the rectangles if the checkbox is ticked
    if !state.draw_rects {
        return;
    }

    // The rotation angle is now affected by the slider value
    let angle = d.get_time() as f32 * state.rotation_speed;

    let center = Vec2::new(d.get_screen_width() as f32, d.get_screen_height() as f32) / 2.0;
    let offset = center / 4.0;

    for i in 0..4 {
        // Changed to 4 for a more symmetrical look
        let rot = glam::Mat2::from_angle(angle + (i as f32 * std::f32::consts::PI / 2.0));
        let rect_pos_rotated = rot * offset + center;

        let size =
            (((d.get_time() as f32 + i as f32 * 1.0) * 2.0).sin() + 1.0) / 2.0 * offset.y + 4.0;

        // Draw the rectangle with a centered origin and the color from the color picker
        d.draw_rectangle_pro(
            Rectangle::new(rect_pos_rotated.x, rect_pos_rotated.y, size, size),
            Vector2::new(size / 2.0, size / 2.0),
            0.0, // The rotation is already in the position, so we don't need to rotate the rect itself
            state.rect_color,
        );
    }
}

/// Draws the GUI controls.
fn draw_gui(state: &mut State, d: &mut RaylibDrawHandle) {
    if state.show_gui_window {
        // The gui_window_box returns `true` if the close button is pressed.
        // We negate the result to keep the window open until it's closed.
        // This is the corrected logic.
        state.show_gui_window =
            !d.gui_window_box(Rectangle::new(20.0, 50.0, 250.0, 300.0), "Controls");

        // --- GUI Controls ---
        // These controls are placed relative to the window box.

        // Checkbox to toggle drawing the rectangles
        d.gui_check_box(
            Rectangle::new(35.0, 90.0, 20.0, 20.0),
            "Draw Rects",
            &mut state.draw_rects,
        );

        // Slider to control rotation speed
        d.gui_slider(
            Rectangle::new(35.0, 120.0, 200.0, 20.0),
            "SLOWER",
            "FASTER",
            &mut state.rotation_speed,
            0.0,
            5.0,
        );

        // Color picker
        state.rect_color = d.gui_color_picker(
            Rectangle::new(35.0, 150.0, 200.0, 100.0),
            "Rect Color",
            state.rect_color,
        );

        // Button to reset all values
        if d.gui_button(Rectangle::new(35.0, 265.0, 200.0, 30.0), "Reset Values") {
            state.rotation_speed = DEFAULT_ROTATION_SPEED;
            state.rect_color = DEFAULT_RECT_COLOR;
            state.draw_rects = DEFAULT_DRAW_RECTS;
        }
    }
}
