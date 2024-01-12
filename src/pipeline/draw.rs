use std::borrow::Cow;

use bevy::{app::Plugin, ecs::{system::{Resource, Commands, Res}, world::{FromWorld, World}, schedule::IntoSystemConfigs}, render::{render_resource::{CachedComputePipelineId, BindGroupLayout, PipelineCache, BindGroupLayoutDescriptor, BindGroupLayoutEntry, ShaderStages, BindingType, StorageTextureAccess, TextureFormat, TextureViewDimension, ComputePipelineDescriptor, PushConstantRange, BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, CachedPipelineState, ComputePassDescriptor}, renderer::{RenderDevice, RenderContext}, Render, render_asset::RenderAssets, texture::Image, RenderSet, render_graph}, asset::AssetServer, math::Vec2};

use crate::{input::DrawingParams, GameOfLifeImage, SIM_SIZE, WORKGROUP_SIZE};

use super::automata::AutomataImageBindGroup;

pub struct DrawPipelinePlugin;

impl Plugin for DrawPipelinePlugin{
    fn build(&self, app: &mut bevy::prelude::App) {
        app
        .init_resource::<DrawPipeline>()
        .add_systems(Render, prepare_bind_group.in_set(RenderSet::PrepareBindGroups));
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DrawPushConstants {
    draw_start: [f32; 2],
    draw_end: [f32; 2],
    draw_radius: f32,
}

impl DrawPushConstants {
    pub fn new(draw_start: Vec2, draw_end: Vec2, draw_radius: f32) -> Self {
        Self {
            draw_radius,
            draw_end: draw_end.to_array(),
            draw_start: draw_start.to_array(),
        }
    }
}


#[derive(Resource)]
pub struct DrawPipeline{
    draw_pipeline: CachedComputePipelineId,
    draw_bind_group_layout: BindGroupLayout,
}


impl FromWorld for DrawPipeline {
    fn from_world(world: &mut World) -> Self {
        let pipeline_cache = world.resource::<PipelineCache>();

        let draw_bind_group_layout =
            world
                .resource::<RenderDevice>()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("Game of Life Bind Group Layout"),
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadWrite,
                            format: TextureFormat::Rgba8Unorm,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    }],
                });

        let brush_shader = world.resource::<AssetServer>().load("shaders/draw.wgsl");

        let draw_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            shader: brush_shader,
            shader_defs: vec![],
            entry_point: Cow::from("draw"), // entry point in shaders file
            layout: vec![draw_bind_group_layout.clone()],
            label: Some(std::borrow::Cow::Borrowed("Game of Life Draw Pipeline")),
            push_constant_ranges: [PushConstantRange {
                stages: ShaderStages::COMPUTE,
                range: 0..std::mem::size_of::<DrawPushConstants>() as u32,
            }]
            .to_vec(),
        });

        DrawPipeline {
            draw_pipeline,
            draw_bind_group_layout,
        }
    }
}


#[derive(Resource)]
struct DrawBindGroup(pub BindGroup);

pub fn prepare_bind_group(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    pipeline: Res<DrawPipeline>,
    gpu_images: Res<RenderAssets<Image>>,
    game_of_life_image: Res<GameOfLifeImage>,
) {
    let view = &gpu_images.get(&game_of_life_image.0).unwrap();
    let draw_bind_group = render_device.create_bind_group(
        Some("Game of Life Draw Bind Group"),
        &pipeline.draw_bind_group_layout,
        &[BindGroupEntry {
            binding: 0,
            resource: BindingResource::TextureView(&view.texture_view),
        }],
    );
    commands.insert_resource(DrawBindGroup(draw_bind_group));
}

// ================================== Nodes ================================== //
pub enum AutomataDrawState {
    Loading,
    Update,
}

pub struct AutomataDrawNode {
    state: AutomataDrawState,
}

impl Default for AutomataDrawNode {
    fn default() -> Self {
        Self {
            state: AutomataDrawState::Loading,
        }
    }
}

impl render_graph::Node for AutomataDrawNode {
    fn update(&mut self, world: &mut World) {
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<DrawPipeline>();

        // if the corresponding pipeline has loaded, transition to the next stage
        match self.state {
            AutomataDrawState::Loading => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.draw_pipeline)
                {
                    self.state = AutomataDrawState::Update;
                }
            }
            AutomataDrawState::Update => {}
        }
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let params = &world.resource::<DrawingParams>();

        if params.is_drawing {
            let texture_bind_group = &world.resource::<AutomataImageBindGroup>().0;
            let draw_bind_group = &world.resource::<AutomataImageBindGroup>().0;
            let pipeline_cache = world.resource::<PipelineCache>();
            let pipeline = world.resource::<DrawPipeline>();

            let mut pass = render_context
                .command_encoder()
                .begin_compute_pass(&ComputePassDescriptor{label: Some("cpd-draw")});

            pass.set_bind_group(0, texture_bind_group, &[]);

            // select the pipeline based on the current state
            match self.state {
                AutomataDrawState::Loading => {}
                AutomataDrawState::Update => {
                    let draw_pipeline = pipeline_cache
                        .get_compute_pipeline(pipeline.draw_pipeline)
                        .unwrap();

                    let pc =
                        DrawPushConstants::new(params.mouse_pos, params.prev_mouse_pos, 10.0);

                    pass.set_pipeline(draw_pipeline);
                    pass.set_bind_group(0, draw_bind_group, &[]);
                    pass.set_push_constants(0, bytemuck::cast_slice(&[pc]));
                    pass.dispatch_workgroups(
                        SIM_SIZE.0 / WORKGROUP_SIZE,
                        SIM_SIZE.1 / WORKGROUP_SIZE,
                        1,
                    );
                }
            }
        }

        Ok(())
    }
}