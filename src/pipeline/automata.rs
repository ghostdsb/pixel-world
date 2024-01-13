use std::borrow::Cow;

use bevy::{ecs::{system::{Resource, Commands, Res}, world::{FromWorld, World}, schedule::IntoSystemConfigs}, prelude::Deref, render::{extract_resource::ExtractResource, texture::Image, RenderSet, render_resource::{CachedComputePipelineId, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, ShaderStages, BindingType, TextureFormat, StorageTextureAccess, TextureViewDimension, PipelineCache, ComputePipelineDescriptor, BindGroup, BindGroupEntries, CachedPipelineState, ComputePassDescriptor}, renderer::{RenderDevice, RenderContext}, render_asset::RenderAssets, render_graph, RenderApp, Render}, asset::{Handle, AssetServer}, app::{Plugin, App, Startup}};

use crate::{SIM_SIZE, WORKGROUP_SIZE, input::DrawingParams};

use super::draw::DrawPipeline;

#[derive(Resource, Clone, Deref, ExtractResource, Debug)]
pub struct GameOfLifeImage(pub Handle<Image>);

pub struct AutomataPipelinePlugin;
impl Plugin for AutomataPipelinePlugin {
    fn build(&self, render_app: &mut App) {
        render_app
            .init_resource::<AutomataPipeline>()
            .init_resource::<DrawPipeline>()
            .add_systems(Render, prepare_bind_group.in_set(RenderSet::PrepareBindGroups));
    }
}


#[derive(Resource)]
pub struct AutomataPipeline {
    init_pipeline: CachedComputePipelineId,
    update_pipeline: CachedComputePipelineId,
    texture_bind_group_layout: BindGroupLayout,
}

impl FromWorld for AutomataPipeline {
    fn from_world(world: &mut World) -> Self {
        let texture_bind_group_layout =
            world.resource::<RenderDevice>()
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

        let pipeline_cache = world.resource::<PipelineCache>();
        
        let shader = world
            .resource::<AssetServer>()
            .load("shaders/game_of_life.wgsl");

        let init_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            shader: shader.clone(),
            shader_defs: vec![],
            layout: vec![texture_bind_group_layout.clone()],
            entry_point: Cow::from("init"),
            push_constant_ranges: Vec::new(),
            label: Some(Cow::Borrowed("Game of Life Init Pipeline")),
        });
        
        let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            shader,
            shader_defs: vec![],
            layout: vec![texture_bind_group_layout.clone()],
            entry_point: Cow::from("update"),
            push_constant_ranges: Vec::new(),
            label: Some(Cow::Borrowed("Game of Life Update Pipeline")),
        });
    
        AutomataPipeline {
            texture_bind_group_layout,
            init_pipeline,
            update_pipeline,
        }
    }
}

#[derive(Resource)]
pub struct AutomataImageBindGroup(pub BindGroup);

pub fn prepare_bind_group(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    pipeline: Res<AutomataPipeline>,
    gpu_images: Res<RenderAssets<Image>>,
    game_of_life_image: Res<GameOfLifeImage>,
) {
    // dbg!(&game_of_life_image);
    let view = &gpu_images.get(&game_of_life_image.0).unwrap();

    let bind_group = render_device.create_bind_group(
         Some("Game of Life Bind Group"),
         &pipeline.texture_bind_group_layout,
         &BindGroupEntries::single(&view.texture_view),
    );
    commands.insert_resource(AutomataImageBindGroup(bind_group));
}

pub enum AutomataState{
    Loading,
    Init,
    Update
}

pub struct AutomataNode{
    state: AutomataState
}

impl Default for AutomataNode{
    fn default() -> Self {
        Self { state: AutomataState::Loading }
    }
}

impl render_graph::Node for AutomataNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<AutomataPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        // if the corresponding pipeline has loaded, transition to the next stage
        match self.state {
            AutomataState::Loading => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.init_pipeline)
                {
                    self.state = AutomataState::Init;
                }
            }
            AutomataState::Init => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.update_pipeline)
                {
                    self.state = AutomataState::Update;
                }
            }
            AutomataState::Update => {}
        }
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        // dbg!(&world.resource::<AutomataImageBindGroup>().0);
        let texture_bind_group = &world.resource::<AutomataImageBindGroup>().0;
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<AutomataPipeline>();

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor{label: Some("cpd-automata")});

        pass.set_bind_group(0, texture_bind_group, &[]);
        let params = &world.resource::<DrawingParams>();
        // select the pipeline based on the current state
        if !params.is_erasing{
            match self.state {
                AutomataState::Loading => {}
                AutomataState::Init => {
                    let init_pipeline = pipeline_cache
                        .get_compute_pipeline(pipeline.init_pipeline)
                        .unwrap();
                    pass.set_pipeline(init_pipeline);
                    pass.dispatch_workgroups(SIM_SIZE.0 / WORKGROUP_SIZE, SIM_SIZE.1 / WORKGROUP_SIZE, 1);
                }
                AutomataState::Update => {
                    let update_pipeline = pipeline_cache
                        .get_compute_pipeline(pipeline.update_pipeline)
                        .unwrap();
                    pass.set_pipeline(update_pipeline);
                    pass.dispatch_workgroups(SIM_SIZE.0 / WORKGROUP_SIZE, SIM_SIZE.1 / WORKGROUP_SIZE, 1);
                }
            }
        }

        Ok(())
    }
}