use bevy::prelude::*;
use bevy::window::WindowMode;
use pixel_world::PixelWorldPlugin; // ToDo: Replace pixel_world with your new crate name.

#[bevy_main]
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resizable: false,
                    mode: WindowMode::BorderlessFullscreen,
                    ..default()
                }),
                ..default()
            }),
            PixelWorldPlugin,
        ))
        .run()
}
