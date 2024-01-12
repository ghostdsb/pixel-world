// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::winit::WinitWindows;
use bevy::DefaultPlugins;
use pixel_world::PixelWorldPlugin; // ToDo: Replace pixel_world with your new crate name.
use std::io::Cursor;
use winit::window::Icon;

mod images;

const SIM_SIZE: (u32, u32) = (1280, 720);
const WORKGROUP_SIZE: u32 = 8;

fn main() {
    App::new()
    .insert_resource(Msaa::Off)
    .insert_resource(AssetMetaCheck::Never)
    .insert_resource(ClearColor(Color::rgb(0.4, 0.4, 0.4)))
    .add_plugins((DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Pixel World".to_string(), // ToDo
            // Bind to canvas included in `index.html`
            canvas: Some("#bevy".to_owned()),
            // The canvas size is constrained in index.html and build/web/styles.css
            fit_canvas_to_parent: true,
            // Tells wasm not to override default event handling, like F5 and Ctrl+R
            prevent_default_event_handling: false,
            present_mode: bevy::window::PresentMode::AutoNoVsync,
            ..default()
        }),
        ..default()
    }), PixelWorldPlugin)
)
    // .add_systems(Startup, setup)
    .add_systems(Startup, set_window_icon)
    .run();
}

// Sets the icon on windows and X11
fn set_window_icon(
    windows: NonSend<WinitWindows>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) {
    let primary_entity = primary_window.single();
    let Some(primary) = windows.get_window(primary_entity) else {
        return;
    };
    let icon_buf = Cursor::new(include_bytes!(
        "../build/macos/AppIcon.iconset/icon_256x256.png"
    ));
    if let Ok(image) = image::load(icon_buf, image::ImageFormat::Png) {
        let image = image.into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        let icon = Icon::from_rgba(rgba, width, height).unwrap();
        primary.set_window_icon(Some(icon));
    };
}

// fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
//     let image = images::create_image(SIM_SIZE.0, SIM_SIZE.1);
//     let image = images.add(image);

//     commands.spawn(SpriteBundle {
//         sprite: Sprite {
//             custom_size: Some(Vec2::new(SIM_SIZE.0 as f32, SIM_SIZE.1 as f32)),
//             ..default()
//         },
//         texture: image.clone(),
//         ..default()
//     });

//     commands.spawn(Camera2dBundle::default());
//     commands.insert_resource(images::GameOfLifeImage(image));
// }