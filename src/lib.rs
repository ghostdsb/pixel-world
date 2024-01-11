#![allow(clippy::type_complexity)]

mod actions;
mod audio;
mod image;
mod loading;
mod menu;
mod player;
mod pipeline;

use crate::actions::ActionsPlugin;
use crate::audio::InternalAudioPlugin;
use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;
use crate::player::PlayerPlugin;

use bevy::{app::App, render::{RenderApp, RenderSet, extract_resource::ExtractResourcePlugin}};
#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use image::GameOfLifeImage;
use pipeline::GameOfLifePipeline;

const SIM_SIZE: (u32, u32) = (1280, 720);
const WORKGROUP_SIZE: u32 = 8;

// This example game uses States to separate logic
// See https://bevy-cheatbook.github.io/programming/states.html
// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    // During the loading State the LoadingPlugin will load our assets
    #[default]
    Loading,
    // During this State the actual game logic is executed
    Playing,
    // Here the menu is drawn and waiting for player interaction
    Menu,
}

pub struct PixelWorldPlugin;

impl Plugin for PixelWorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>()
            .add_systems(Startup, setup)
            .add_plugins((
                LoadingPlugin,
                // MenuPlugin,
                ActionsPlugin,
                InternalAudioPlugin,
                // PlayerPlugin,
            ));

        #[cfg(debug_assertions)]
        {
            app.add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()));
        }
    }

    fn finish(&self, app: &mut App) {
        let render_app = match app.get_sub_app_mut(RenderApp) {
            Ok(render_app) => render_app,
            Err(_) => return,
        };
        render_app
        .init_resource::<GameOfLifePipeline>()
        .add_systems(Startup, pipeline::queue_bind_group.in_set(RenderSet::Queue))
        .add_plugins(ExtractResourcePlugin::<GameOfLifeImage>::default());
    }
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let image = image::create_image(SIM_SIZE.0, SIM_SIZE.1);
    let image = images.add(image);

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(SIM_SIZE.0 as f32, SIM_SIZE.1 as f32)),
            ..default()
        },
        texture: image.clone(),
        ..default()
    });

    commands.spawn(Camera2dBundle::default());
    commands.insert_resource(image::GameOfLifeImage(image));
}
