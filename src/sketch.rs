use raylib::prelude::*;

pub const FRAMES_PER_SECOND: u32 = 60;
const CAMERA_SPEED: f32 = 400.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shape {
    Square,
    Circle,
    Triangle,
}

/// Represents a single object in our scene tree.
#[derive(Debug, Clone)]
pub struct SceneObject {
    pub id: u32,
    pub text: String,
    pub shape: Shape,
    pub color: Color,
    pub position: Vector2,
    pub rotation_speed: f32,
    pub current_rotation: f32,
    pub children: Vec<SceneObject>,
    // GUI-specific state
    pub show_settings: bool,
    pub text_buffer: String,
}

impl SceneObject {
    /// Helper to create a new object.
    fn new(id: u32, text: &str, shape: Shape, color: Color, position: Vector2) -> Self {
        Self {
            id,
            text: text.to_string(),
            shape,
            color,
            position,
            rotation_speed: 20.0,
            current_rotation: 0.0,
            children: Vec::new(),
            show_settings: false,
            text_buffer: text.to_string(),
        }
    }
}

/// The main state for the application.
pub struct State {
    pub time_since_last_update: f32,
    pub scene_objects: Vec<SceneObject>,
    pub camera: Camera2D,
    next_id: u32,
}

impl State {
    pub fn new() -> Self {
        let mut state = Self {
            time_since_last_update: 0.0,
            scene_objects: Vec::new(),
            camera: Camera2D {
                target: Vector2::new(640.0, 360.0),
                offset: Vector2::new(1280.0 / 2.0, 720.0 / 2.0),
                rotation: 0.0,
                zoom: 1.0,
            },
            next_id: 0,
        };

        // --- Create a sample scene tree ---
        let mut root = SceneObject::new(
            state.new_id(),
            "Root",
            Shape::Square,
            Color::RED,
            Vector2::new(640.0, 100.0),
        );
        let mut child1 = SceneObject::new(
            state.new_id(),
            "Child A",
            Shape::Circle,
            Color::BLUE,
            Vector2::new(400.0, 300.0),
        );
        let child2 = SceneObject::new(
            state.new_id(),
            "Child B",
            Shape::Triangle,
            Color::GREEN,
            Vector2::new(880.0, 300.0),
        );
        let grandchild = SceneObject::new(
            state.new_id(),
            "Grandchild",
            Shape::Square,
            Color::YELLOW,
            Vector2::new(400.0, 500.0),
        );

        child1.children.push(grandchild);
        root.children.push(child1);
        root.children.push(child2);
        state.scene_objects.push(root);

        state
    }

    /// Generates a new unique ID for scene objects.
    fn new_id(&mut self) -> u32 {
        self.next_id += 1;
        self.next_id
    }
}

/// Handles user input for camera movement.
pub fn process_events_and_input(rl: &mut RaylibHandle, state: &mut State) {
    let dt = rl.get_frame_time();
    let move_amount = CAMERA_SPEED * dt;
    if rl.is_key_down(KeyboardKey::KEY_UP) {
        state.camera.target.y -= move_amount;
    }
    if rl.is_key_down(KeyboardKey::KEY_DOWN) {
        state.camera.target.y += move_amount;
    }
    if rl.is_key_down(KeyboardKey::KEY_LEFT) {
        state.camera.target.x -= move_amount;
    }
    if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
        state.camera.target.x += move_amount;
    }
}

/// Updates the state of all objects in the scene.
pub fn step(state: &mut State, dt: f32) {
    for obj in &mut state.scene_objects {
        update_object_recursively(obj, dt);
    }
}

/// Helper function to recursively update an object and its children.
fn update_object_recursively(obj: &mut SceneObject, dt: f32) {
    obj.current_rotation += obj.rotation_speed * dt;
    for child in &mut obj.children {
        update_object_recursively(child, dt);
    }
}

/// Main drawing function.
pub fn draw(state: &mut State, d: &mut RaylibDrawHandle) {
    // --- World-space drawing ---
    {
        let mut d2d = d.begin_mode2D(state.camera);
        for obj in &state.scene_objects {
            draw_object_recursively(&mut d2d, obj);
        }
    } // End 2D mode

    // --- Screen-space GUI drawing ---
    // We must iterate through the objects mutably to change their GUI state.
    for obj in &mut state.scene_objects {
        draw_gui_recursively(d, obj, &state.camera);
    }

    d.draw_text("Use Arrow Keys to Pan Camera", 10, 10, 20, Color::WHITE);
}

/// Recursively draws an object, its connection lines, and its children in world space.
fn draw_object_recursively(d: &mut RaylibMode2D<RaylibDrawHandle>, obj: &SceneObject) {
    // Draw lines to children first, so they appear underneath the shapes.
    for child in &obj.children {
        d.draw_line_v(obj.position, child.position, Color::GRAY);
        draw_object_recursively(d, child);
    }

    // Draw the object's shape
    let size = 40.0;
    match obj.shape {
        Shape::Square => d.draw_rectangle_pro(
            Rectangle::new(obj.position.x, obj.position.y, size, size),
            Vector2::new(size / 2.0, size / 2.0),
            obj.current_rotation,
            obj.color,
        ),
        Shape::Circle => d.draw_circle_v(obj.position, size / 2.0, obj.color),
        Shape::Triangle => d.draw_triangle(
            Vector2::new(obj.position.x, obj.position.y - size / 2.0),
            Vector2::new(obj.position.x - size / 2.0, obj.position.y + size / 2.0),
            Vector2::new(obj.position.x + size / 2.0, obj.position.y + size / 2.0),
            obj.color,
        ),
    }

    // Draw the text label
    let text_size = d
        .get_font_default()
        .measure_text(obj.text.as_str(), 20.0, 1.0);
    d.draw_text(
        obj.text.as_str(),
        (obj.position.x - text_size.x / 2.0) as i32,
        (obj.position.y + 30.0) as i32,
        20,
        Color::WHITE,
    );
}

/// Manually calculates the screen position of a world coordinate point.
/// This is the forward camera transformation.
fn world_to_screen(world_pos: Vector2, camera: &Camera2D) -> Vector2 {
    (world_pos - camera.target) * camera.zoom + camera.offset
}

/// Recursively draws the GUI for an object and its children in screen space.
fn draw_gui_recursively(d: &mut RaylibDrawHandle, obj: &mut SceneObject, camera: &Camera2D) {
    // CORRECTED: Manually calculate the screen position using our helper function.
    let screen_pos = world_to_screen(obj.position, camera);

    // Draw the settings button for this object
    let button_rect = Rectangle::new(screen_pos.x + 30.0, screen_pos.y, 30.0, 20.0);
    if d.gui_button(button_rect, "[S]") {
        obj.show_settings = !obj.show_settings;
    }

    // If settings are shown, draw the window and its controls
    if obj.show_settings {
        let window_rect = Rectangle::new(screen_pos.x + 70.0, screen_pos.y, 250.0, 220.0);
        obj.show_settings = !d.gui_window_box(window_rect, &format!("Settings: {}", obj.text));

        // --- Controls within the window ---
        let base_x = window_rect.x + 10.0;
        let base_y = window_rect.y + 30.0;

        // Text Box
        if d.gui_text_box(
            Rectangle::new(base_x, base_y, 230.0, 30.0),
            &mut obj.text_buffer,
            true,
        ) {
            obj.text = obj.text_buffer.clone();
        }

        // Shape Toggle
        let shape_text = "Square;Circle;Triangle";
        let mut active_shape = obj.shape as i32;
        d.gui_toggle_group(
            Rectangle::new(base_x, base_y + 40.0, 70.0, 25.0),
            shape_text,
            &mut active_shape,
        );
        obj.shape = match active_shape {
            0 => Shape::Square,
            1 => Shape::Circle,
            _ => Shape::Triangle,
        };

        // Rotation Speed Slider
        d.gui_slider_bar(
            Rectangle::new(base_x, base_y + 75.0, 230.0, 20.0),
            "Spin",
            &format!("{:.0}", obj.rotation_speed),
            &mut obj.rotation_speed,
            -180.0,
            180.0,
        );

        // Color Picker
        obj.color = d.gui_color_picker(
            Rectangle::new(base_x, base_y + 105.0, 230.0, 80.0),
            "",
            obj.color,
        );
    }

    // Recurse for children
    for child in &mut obj.children {
        draw_gui_recursively(d, child, camera);
    }
}
