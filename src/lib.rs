#![allow(clippy::type_complexity)]

mod actions;
mod audio;
mod images;
mod loading;
mod menu;
mod player;
mod pipeline;
mod camera;
mod input;

use crate::actions::ActionsPlugin;
use crate::audio::InternalAudioPlugin;
use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;
use crate::player::PlayerPlugin;

use bevy::{app::App, render::{RenderApp, RenderSet, extract_resource::ExtractResourcePlugin, render_graph::RenderGraph, Render, camera::CameraPlugin}};
#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use input::DrawingParams;
use pipeline::{automata::{GameOfLifeImage, prepare_bind_group}, PipelinesPlugin};
// use pipeline::{GameOfLifePipeline, GameOfLifeNode, prepare_bind_group};

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
       // Extract the game of life image resource from the main world into the render world
        // for operation on by the compute shader and display on the sprite.
        app
        .add_systems(Startup, setup)
        .add_plugins(ExtractResourcePlugin::<GameOfLifeImage>::default())
        .add_plugins(ExtractResourcePlugin::<DrawingParams>::default())
        .add_plugins(camera::CameraPlugin)
        .add_plugins(input::InputPlugin)
        .add_plugins(PipelinesPlugin);
        
        #[cfg(debug_assertions)]
        {
            app.add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()));
        }
    }

    // fn finish(&self, app: &mut App) {
    //     let render_app = app.sub_app_mut(RenderApp);
    //     // render_app.init_resource::<GameOfLifePipeline>();
    // }
}

// add this in build, and first system on startup
fn setup(mut commands: Commands, mut images_res: ResMut<Assets<Image>>) {
    let image = images::create_image(SIM_SIZE.0, SIM_SIZE.1);
    let image = images_res.add(image);

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(SIM_SIZE.0 as f32, SIM_SIZE.1 as f32)),
            ..default()
        },
        texture: image.clone(),
        ..default()
    });

    commands.spawn(Camera2dBundle::default());
    commands.insert_resource(pipeline::automata::GameOfLifeImage(image));
}
