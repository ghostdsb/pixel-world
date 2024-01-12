use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
    render::extract_resource::ExtractResource,
};

pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DrawingParams>()
            .add_systems(Update, update_input_state);
    }
}

#[derive(Default, Resource, ExtractResource, Clone)]
pub struct DrawingParams {
    pub mouse_pos: Vec2,
    pub is_drawing: bool,
    pub prev_mouse_pos: Vec2,
}

pub fn update_input_state(
    window_query: Query<&Window>,
    mut input_state: ResMut<DrawingParams>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
) {
    let Ok(primary_window) = window_query.get_single() else { return };
    // get the camera info and transform
    let Ok((camera, camera_transform)) = camera_q.get_single() else { return };

    // Determine button state
    for event in mouse_button_input_events.read() {
        if event.button == MouseButton::Left {
            input_state.is_drawing = event.state == ButtonState::Pressed;
        }
    }

    if let Some(world_position) = primary_window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        input_state.prev_mouse_pos = input_state.mouse_pos;
        input_state.mouse_pos =
            world_pos_to_canvas_pos(world_position * Vec2::new(1.0, -1.0));
    }
}

fn world_pos_to_canvas_pos(world_pos: Vec2) -> Vec2 {
    world_pos
        + Vec2::new(
            crate::SIM_SIZE.0 as f32 / 2.0,
            crate::SIM_SIZE.1 as f32 / 2.0,
        )
}