use bevy::{app::{Plugin, Startup, Update}, time::Time, ecs::{system::{Res, Query}, event::EventReader, query::With}, input::{Input, keyboard::KeyCode, mouse::{MouseWheel, MouseScrollUnit}}, transform::components::Transform, render::camera::{OrthographicProjection, Camera}, math::Vec2};

const CAMERA_MOVE_SPEED: f32 = 500.0;

pub struct CameraPlugin;

impl Plugin for CameraPlugin{
    fn build(&self, app: &mut bevy::prelude::App) {
        app
        .add_systems(Update, camera_controller);
    }
}

fn camera_controller(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query: Query<(&mut Transform, &mut OrthographicProjection), With<Camera>>,
) 
{
    for (mut transform, mut ortho) in query.iter_mut() {
        let up = keyboard_input.pressed(KeyCode::W);
        let down = keyboard_input.pressed(KeyCode::S);
        let left = keyboard_input.pressed(KeyCode::A);
        let right = keyboard_input.pressed(KeyCode::D);

        let x_axis = right as i8 - left as i8;
        let y_axis = up as i8 - down as i8;
        let mut move_delta = Vec2::new(x_axis as f32, y_axis as f32);

        // =========== Move the camera around =========== //
        if move_delta != Vec2::ZERO {
            move_delta /= move_delta.length();

            let z = transform.translation.z;
            transform.translation +=
                move_delta.extend(z) * CAMERA_MOVE_SPEED * time.delta_seconds();

            // Important! We need to restore the Z values when moving the camera around.
            // Bevy has a specific camera setup and this can mess with how our layers are shown.
            transform.translation.z = z;
        }

        // =========== Zoom =========== //
        for MouseWheel { x, y, unit, window } in mouse_wheel_events.read() {
            let mut x_scroll_diff = 0.0;
            let mut y_scroll_diff = 0.0;

            match unit {
                MouseScrollUnit::Line => {
                    x_scroll_diff += x;
                    y_scroll_diff += y;
                }
                MouseScrollUnit::Pixel => {
                    // I just took this from three-rs, no idea why this magic number was chosen ¯\_(ツ)_/¯
                    const PIXELS_PER_LINE: f32 = 38.0;

                    y_scroll_diff += y / PIXELS_PER_LINE;
                    x_scroll_diff += x / PIXELS_PER_LINE;
                }
            }

            if x_scroll_diff != 0.0 || y_scroll_diff != 0.0 {
                if y_scroll_diff < 0.0 {
                    ortho.scale *= 1.05;
                } else {
                    ortho.scale *= 1.0 / 1.05;
                }

                ortho.scale = ortho.scale.clamp(0.15, 5.);
            }
        }
    }
}