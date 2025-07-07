use glam::Vec2;
use raylib::prelude::*;

pub const FRAMES_PER_SECOND: u32 = 60;

// Default values for our GUI controls
const DEFAULT_ROTATION_SPEED: f32 = 1.0;
const DEFAULT_RECT_COLOR: Color = Color::RED;
const DEFAULT_DRAW_RECTS: bool = true;
const RENDER_TEXTURE_SIZE: u32 = 400;

pub struct State {
    pub running: bool,
    pub time_since_last_update: f32,

    // --- GUI State ---
    pub show_gui_window: bool,
    pub rotation_speed: f32,
    pub draw_rects: bool,
    pub rect_color: Color,
    /// A buffer to hold text from a text box.
    pub text_box_buffer: String,

    // --- Render Texture ---
    /// The off-screen canvas for our sketch.
    pub render_texture: RenderTexture2D,
}

impl State {
    /// Note: State::new now needs a &mut RaylibHandle and &RaylibThread to create the texture.
    pub fn new(rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        Self {
            running: true,
            time_since_last_update: 0.0,
            show_gui_window: true,
            rotation_speed: DEFAULT_ROTATION_SPEED,
            draw_rects: DEFAULT_DRAW_RECTS,
            rect_color: DEFAULT_RECT_COLOR,
            // Pre-allocate some space for the text buffer
            text_box_buffer: String::with_capacity(64),
            // Load the render texture
            render_texture: rl
                .load_render_texture(thread, RENDER_TEXTURE_SIZE, RENDER_TEXTURE_SIZE)
                .unwrap(),
        }
    }
}

pub fn process_events_and_input(rl: &mut RaylibHandle, state: &mut State) {
    if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_ESCAPE) {
        state.running = false;
    }
    if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_G) {
        state.show_gui_window = !state.show_gui_window;
    }
}

pub fn step(_rl: &mut RaylibHandle, _rlt: &mut RaylibThread, _state: &mut State) {
    // The fixed-step update loop remains separate for good practice,
    // even if it's empty in this particular demo.
}

/// Main drawing function, which calls our sketch and GUI drawing functions.
pub fn draw(state: &mut State, d: &mut RaylibDrawHandle, thread: &RaylibThread) {
    // 1. Draw the sketch into the off-screen render texture.
    // We wrap this in a block to limit the lifetime of the `d_texture` borrow.
    {
        let mut d_texture = d.begin_texture_mode(thread, &mut state.render_texture);
        // Pass the required values directly, instead of the whole state struct,
        // to avoid conflicting borrows.
        draw_sketch_to_texture(
            &mut d_texture,
            state.draw_rects,
            state.rotation_speed,
            state.rect_color,
        );
    } // `d_texture` is dropped here, releasing the mutable borrow on `d` and `state`.

    // 2. Now we can safely draw to the main screen again.
    d.clear_background(Color::DARKGRAY);
    draw_gui_and_texture(state, d);

    // Draw some helper text on the main screen
    d.draw_text("Press 'G' to toggle GUI", 12, 12, 20, Color::WHITE);
    let mouse_pos = d.get_mouse_position();
    d.draw_circle(mouse_pos.x as i32, mouse_pos.y as i32, 6.0, Color::GREEN);
}

/// Draws the spinning rectangles into the provided draw handle (which points to our texture).
/// This function no longer needs the whole state, just the values for drawing.
fn draw_sketch_to_texture(
    d: &mut RaylibTextureMode<RaylibDrawHandle>,
    draw_rects: bool,
    rotation_speed: f32,
    rect_color: Color,
) {
    // Clear the texture with a different background to distinguish it
    d.clear_background(Color::BLACK);

    if !draw_rects {
        return;
    }

    let angle = d.get_time() as f32 * rotation_speed;

    // IMPORTANT: All coordinates are now relative to the texture's size.
    let center = Vec2::new(RENDER_TEXTURE_SIZE as f32, RENDER_TEXTURE_SIZE as f32) / 2.0;
    let offset = center / 4.0;

    for i in 0..4 {
        let rot = glam::Mat2::from_angle(angle + (i as f32 * std::f32::consts::PI / 2.0));
        let rect_pos_rotated = rot * offset + center;
        let size =
            (((d.get_time() as f32 + i as f32 * 1.0) * 2.0).sin() + 1.0) / 2.0 * offset.y + 4.0;

        d.draw_rectangle_pro(
            Rectangle::new(rect_pos_rotated.x, rect_pos_rotated.y, size, size),
            Vector2::new(size / 2.0, size / 2.0),
            0.0,
            rect_color,
        );
    }
}

/// Draws the GUI controls and the final render texture to the screen.
fn draw_gui_and_texture(state: &mut State, d: &mut RaylibDrawHandle) {
    // --- Draw the Render Texture to the screen ---
    // We draw it as a simple texture. Note that the Y-axis is flipped in textures.
    let texture_x = 300.0;
    let texture_y = 50.0;
    d.draw_texture_rec(
        state.render_texture.texture(),
        Rectangle::new(
            0.0,
            0.0,
            RENDER_TEXTURE_SIZE as f32,
            -(RENDER_TEXTURE_SIZE as f32),
        ), // Negative height flips it back upright
        Vector2::new(texture_x, texture_y),
        Color::WHITE,
    );
    // Draw a border around it so we can see its bounds.
    d.draw_rectangle_lines(
        texture_x as i32,
        texture_y as i32,
        RENDER_TEXTURE_SIZE as i32,
        RENDER_TEXTURE_SIZE as i32,
        Color::RAYWHITE,
    );

    // --- Draw the GUI Window ---
    if state.show_gui_window {
        state.show_gui_window = !d.gui_window_box(
            Rectangle::new(20.0, 50.0, 250.0, 400.0), // Made window taller
            "Controls",
        );

        // --- GUI Controls ---
        d.gui_check_box(
            Rectangle::new(35.0, 90.0, 20.0, 20.0),
            "Draw Rects",
            &mut state.draw_rects,
        );
        d.gui_slider(
            Rectangle::new(35.0, 120.0, 200.0, 20.0),
            "SLOWER",
            "FASTER",
            &mut state.rotation_speed,
            0.0,
            5.0,
        );
        state.rect_color = d.gui_color_picker(
            Rectangle::new(35.0, 150.0, 200.0, 100.0),
            "Rect Color",
            state.rect_color,
        );

        // Text Box for typing
        d.gui_label(Rectangle::new(35.0, 260.0, 200.0, 20.0), "Enter Text:");
        d.gui_text_box(
            Rectangle::new(35.0, 280.0, 200.0, 30.0),
            &mut state.text_box_buffer,
            true,
        );

        // Display the text from the buffer
        let display_text = format!("You typed: {}", state.text_box_buffer);
        d.gui_label(Rectangle::new(35.0, 320.0, 200.0, 20.0), &display_text);

        // Button to reset all values
        if d.gui_button(Rectangle::new(35.0, 360.0, 200.0, 30.0), "Reset Values") {
            state.rotation_speed = DEFAULT_ROTATION_SPEED;
            state.rect_color = DEFAULT_RECT_COLOR;
            state.draw_rects = DEFAULT_DRAW_RECTS;
            state.text_box_buffer.clear();
        }
    }
}
