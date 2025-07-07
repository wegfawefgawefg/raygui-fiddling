use raylib::prelude::*;
use std::collections::HashMap;

pub const FRAMES_PER_SECOND: u32 = 60;

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
    pub rotation_speed: f32,
    pub current_rotation: f32,
    pub children: Vec<SceneObject>,
    pub text_buffer: String,
}

impl SceneObject {
    fn new(id: u32, text: &str, shape: Shape, color: Color) -> Self {
        // Pre-allocate the buffer with enough capacity for edits.
        let mut text_buffer = String::with_capacity(128);
        text_buffer.push_str(text);

        Self {
            id,
            text: text.to_string(),
            shape,
            color,
            rotation_speed: 20.0,
            current_rotation: 0.0,
            children: Vec::new(),
            text_buffer,
        }
    }
}

/// A request to modify the scene tree, processed at the end of the frame.
#[derive(Debug, Clone)]
pub enum EditorRequest {
    AddChild { parent_id: u32 },
    DeleteNode { node_id: u32 },
}

/// The main state for the application.
pub struct State {
    pub time_since_last_update: f32,
    pub scene_objects: Vec<SceneObject>,
    pub camera: Camera2D,
    next_id: u32,
    // The ID of the one object whose full settings are being shown.
    pub active_settings_id: Option<u32>,
    // The ID of the text box that was active on the previous frame.
    pub last_active_textbox_id: Option<u32>,
    // A queue for add/delete operations.
    pub requests: Vec<EditorRequest>,
}

impl State {
    pub fn new() -> Self {
        let mut state = Self {
            time_since_last_update: 0.0,
            scene_objects: Vec::new(),
            camera: Camera2D {
                target: Vector2::new(400.0, 450.0),
                offset: Vector2::new(1280.0 / 2.0, 720.0 / 2.0),
                rotation: 0.0,
                zoom: 1.0,
            },
            next_id: 0,
            active_settings_id: None,
            last_active_textbox_id: None,
            requests: Vec::new(),
        };

        // --- Create a sample scene tree ---
        let mut root = SceneObject::new(state.new_id(), "Root", Shape::Square, Color::RED);
        let mut child1 = SceneObject::new(state.new_id(), "Data", Shape::Circle, Color::BLUE);
        let mut child2 = SceneObject::new(state.new_id(), "Render", Shape::Triangle, Color::GREEN);
        let grandchild1a = SceneObject::new(state.new_id(), "Mesh", Shape::Square, Color::YELLOW);
        let grandchild1b =
            SceneObject::new(state.new_id(), "Texture", Shape::Triangle, Color::ORANGE);
        let grandchild2a = SceneObject::new(state.new_id(), "Shader", Shape::Circle, Color::PURPLE);

        child1.children.push(grandchild1a);
        child1.children.push(grandchild1b);
        child2.children.push(grandchild2a);
        root.children.push(child1);
        root.children.push(child2);
        state.scene_objects.push(root);

        state
    }

    pub fn new_id(&mut self) -> u32 {
        self.next_id += 1;
        self.next_id
    }
}

/// Handles user input for camera movement.
pub fn process_events_and_input(rl: &mut RaylibHandle, state: &mut State) {
    if state.last_active_textbox_id.is_some() {
        return;
    } // Don't pan/zoom while typing

    const ZOOM_INCREMENT: f32 = 0.125;
    const MIN_ZOOM: f32 = 0.1;
    const MAX_ZOOM: f32 = 2.0;

    let wheel_move = rl.get_mouse_wheel_move();
    if wheel_move != 0.0 {
        let wheel_direction = if wheel_move > 0.0 { 1.0 } else { -1.0 };
        state.camera.zoom += wheel_direction * ZOOM_INCREMENT;
    }
    if rl.is_key_down(KeyboardKey::KEY_EQUAL) {
        state.camera.zoom += 1.0 * rl.get_frame_time();
    }
    if rl.is_key_down(KeyboardKey::KEY_MINUS) {
        state.camera.zoom -= 1.0 * rl.get_frame_time();
    }
    state.camera.zoom = state.camera.zoom.clamp(MIN_ZOOM, MAX_ZOOM);

    if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT) {
        let delta = rl.get_mouse_delta() / state.camera.zoom;
        state.camera.target -= delta;
    }
}

/// Updates the state of all objects in the scene.
pub fn step(state: &mut State, dt: f32) {
    for obj in &mut state.scene_objects {
        update_object_recursively(obj, dt);
    }

    // Process any pending add/delete requests.
    process_editor_requests(state);
}

fn update_object_recursively(obj: &mut SceneObject, dt: f32) {
    obj.current_rotation += obj.rotation_speed * dt;
    for child in &mut obj.children {
        update_object_recursively(child, dt);
    }
}

/// Main drawing function.
pub fn draw(state: &mut State, d: &mut RaylibDrawHandle) {
    let mut layout_positions = HashMap::new();

    {
        let mut d2d = d.begin_mode2D(state.camera);

        let mut start_y = 100.0;
        for root_obj in &state.scene_objects {
            layout_recursive(
                root_obj,
                200.0,
                start_y,
                &mut start_y,
                &mut layout_positions,
            );
        }

        // Collect all IDs to iterate over, avoiding borrow checker issues.
        let all_ids = layout_positions.keys().cloned().collect::<Vec<_>>();
        for id in all_ids {
            // CORRECTED: Removed the screen-space draw handle from this call.
            draw_object_and_gui(&mut d2d, state, id, &layout_positions);
        }
    }

    d.draw_text(
        "Right Mouse to Pan, Scroll Wheel to Zoom",
        10,
        10,
        20,
        Color::WHITE,
    );
}

/// The layout algorithm. Calculates positions for a left-to-right tree.
fn layout_recursive(
    obj: &SceneObject,
    x: f32,
    y_start: f32,
    y_cursor: &mut f32,
    positions: &mut HashMap<u32, Vector2>,
) -> f32 {
    const X_SPACING: f32 = 350.0;
    const Y_SPACING: f32 = 180.0;

    let mut children_height = 0.0;
    let mut child_y_cursor = y_start;

    for child in &obj.children {
        children_height += layout_recursive(
            child,
            x + X_SPACING,
            child_y_cursor,
            &mut child_y_cursor,
            positions,
        );
    }

    let my_pos = if !obj.children.is_empty() {
        let my_y = y_start + (children_height / 2.0) - (Y_SPACING / 2.0);
        Vector2::new(x, my_y)
    } else {
        Vector2::new(x, *y_cursor)
    };

    positions.insert(obj.id, my_pos);

    let my_height = if children_height > 0.0 {
        children_height
    } else {
        Y_SPACING
    };

    *y_cursor = y_start + my_height;

    my_height
}

/// Draws a single object and its GUI. This function is called for every node.
fn draw_object_and_gui(
    d_world: &mut RaylibMode2D<RaylibDrawHandle>, // The world-space (camera) draw handle
    state: &mut State,
    obj_id: u32,
    positions: &HashMap<u32, Vector2>,
) {
    let obj_clone = match find_object_by_id_mut(&mut state.scene_objects, obj_id) {
        Some(o) => o.clone(),
        None => return,
    };

    let my_pos = positions.get(&obj_id).cloned().unwrap_or_default();

    // Draw lines to children
    for child in &obj_clone.children {
        if let Some(child_pos) = positions.get(&child.id) {
            d_world.draw_line_v(my_pos, *child_pos, Color::GRAY);
        }
    }

    // --- Draw the object's shape in WORLD space ---
    let size = 40.0;
    match obj_clone.shape {
        Shape::Square => d_world.draw_rectangle_pro(
            Rectangle::new(my_pos.x, my_pos.y, size, size),
            Vector2::new(size / 2.0, size / 2.0),
            obj_clone.current_rotation,
            obj_clone.color,
        ),
        Shape::Circle => d_world.draw_circle_v(my_pos, size / 2.0, obj_clone.color),
        Shape::Triangle => {
            let angle_rad = obj_clone.current_rotation.to_radians();
            let cos_a = angle_rad.cos();
            let sin_a = angle_rad.sin();
            let p1 = Vector2::new(0.0, -size / 2.0);
            let p2 = Vector2::new(-size / 2.0, size / 2.0);
            let p3 = Vector2::new(size / 2.0, size / 2.0);
            let rp1 =
                Vector2::new(p1.x * cos_a - p1.y * sin_a, p1.x * sin_a + p1.y * cos_a) + my_pos;
            let rp2 =
                Vector2::new(p2.x * cos_a - p2.y * sin_a, p2.x * sin_a + p2.y * cos_a) + my_pos;
            let rp3 =
                Vector2::new(p3.x * cos_a - p3.y * sin_a, p3.x * sin_a + p3.y * cos_a) + my_pos;
            d_world.draw_triangle(rp1, rp2, rp3, obj_clone.color);
        }
    }

    // --- Draw Labels and GUIs (all scaled) in WORLD space ---
    let scale = 1.0 / state.camera.zoom;
    let font = d_world.get_font_default();
    let text_size = font.measure_text(&obj_clone.text, 20.0 * scale, 1.0 * scale);
    // CORRECTED: Cast font size to i32.
    d_world.draw_text(
        &obj_clone.text,
        (my_pos.x - text_size.x / 2.0) as i32,
        (my_pos.y + 30.0 * scale) as i32,
        (20.0 * scale) as i32,
        Color::WHITE,
    );

    // --- Always-on Details Panel ---
    let details_rect = Rectangle::new(
        my_pos.x - 50.0 * scale,
        my_pos.y + 60.0 * scale,
        100.0 * scale,
        55.0 * scale,
    );
    // CORRECTED: Removed `Some()` wrapper.
    d_world.gui_group_box(details_rect, "Details");
    d_world.gui_label(
        Rectangle::new(
            details_rect.x + 5.0 * scale,
            details_rect.y + 10.0 * scale,
            80.0 * scale,
            20.0 * scale,
        ),
        &format!("Shape: {:?}", obj_clone.shape),
    );
    d_world.draw_rectangle_rec(
        Rectangle::new(
            details_rect.x + 5.0 * scale,
            details_rect.y + 30.0 * scale,
            90.0 * scale,
            15.0 * scale,
        ),
        obj_clone.color,
    );

    // --- Settings Button ---
    if d_world.gui_button(
        Rectangle::new(
            my_pos.x + 30.0 * scale,
            my_pos.y,
            30.0 * scale,
            20.0 * scale,
        ),
        "[S]",
    ) {
        state.active_settings_id = Some(obj_id);
    }

    // --- Full Settings Panel (if active) ---
    if state.active_settings_id == Some(obj_id) {
        if let Some(obj) = find_object_by_id_mut(&mut state.scene_objects, obj_id) {
            let window_rect = Rectangle::new(
                my_pos.x + 70.0 * scale,
                my_pos.y,
                250.0 * scale,
                250.0 * scale,
            );
            if d_world.gui_window_box(window_rect, &format!("Settings: {}", obj.text)) {
                state.active_settings_id = None;
            }

            let base_x = window_rect.x + 10.0 * scale;
            let base_y = window_rect.y + 30.0 * scale;

            let textbox_rect = Rectangle::new(base_x, base_y, 230.0 * scale, 30.0 * scale);
            let is_active = d_world.gui_text_box(textbox_rect, &mut obj.text_buffer, true);

            if let Some(nul_pos) = obj.text_buffer.find('\0') {
                obj.text_buffer.truncate(nul_pos);
            }

            if is_active {
                state.last_active_textbox_id = Some(obj.id);
            } else if state.last_active_textbox_id == Some(obj.id) {
                obj.text = obj.text_buffer.clone();
                state.last_active_textbox_id = None;
            }

            let mut active_shape = obj.shape as i32;
            d_world.gui_toggle_group(
                Rectangle::new(base_x, base_y + 40.0 * scale, 70.0 * scale, 25.0 * scale),
                "Square;Circle;Triangle",
                &mut active_shape,
            );
            obj.shape = match active_shape {
                0 => Shape::Square,
                1 => Shape::Circle,
                _ => Shape::Triangle,
            };

            // CORRECTED: `gui_slider_bar` expects &str for text, not Option.
            d_world.gui_slider_bar(
                Rectangle::new(base_x, base_y + 75.0 * scale, 230.0 * scale, 20.0 * scale),
                "",
                &format!("{:.0}", obj.rotation_speed),
                &mut obj.rotation_speed,
                -180.0,
                180.0,
            );

            obj.color = d_world.gui_color_picker(
                Rectangle::new(base_x, base_y + 105.0 * scale, 230.0 * scale, 80.0 * scale),
                "",
                obj.color,
            );

            if d_world.gui_button(
                Rectangle::new(base_x, base_y + 195.0 * scale, 110.0 * scale, 20.0 * scale),
                "Add Child",
            ) {
                state
                    .requests
                    .push(EditorRequest::AddChild { parent_id: obj.id });
            }
            if d_world.gui_button(
                Rectangle::new(
                    base_x + 120.0 * scale,
                    base_y + 195.0 * scale,
                    110.0 * scale,
                    20.0 * scale,
                ),
                "Delete Node",
            ) {
                state
                    .requests
                    .push(EditorRequest::DeleteNode { node_id: obj.id });
                state.active_settings_id = None;
            }
        }
    }
}

/// Processes the request queue to add or delete nodes.
fn process_editor_requests(state: &mut State) {
    let requests = state.requests.drain(..).collect::<Vec<_>>();
    for req in requests {
        match req {
            EditorRequest::AddChild { parent_id } => {
                let new_id = state.new_id();
                if let Some(parent) = find_object_by_id_mut(&mut state.scene_objects, parent_id) {
                    let new_node =
                        SceneObject::new(new_id, "New Node", Shape::Square, Color::WHITE);
                    parent.children.push(new_node);
                }
            }
            EditorRequest::DeleteNode { node_id } => {
                find_and_delete_node(&mut state.scene_objects, node_id);
            }
        }
    }
}

fn find_and_delete_node(objects: &mut Vec<SceneObject>, id_to_delete: u32) -> bool {
    if let Some(index) = objects.iter().position(|o| o.id == id_to_delete) {
        objects.remove(index);
        return true;
    }
    for obj in objects {
        if find_and_delete_node(&mut obj.children, id_to_delete) {
            return true;
        }
    }
    false
}

fn find_object_by_id_mut<'a>(
    objects: &'a mut [SceneObject],
    id: u32,
) -> Option<&'a mut SceneObject> {
    for obj in objects {
        if obj.id == id {
            return Some(obj);
        }
        if let Some(found) = find_object_by_id_mut(&mut obj.children, id) {
            return Some(found);
        }
    }
    None
}
