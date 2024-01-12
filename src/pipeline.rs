use std::borrow::Cow;

use bevy::{prelude::*, render::{render_resource::*, renderer::{RenderDevice, RenderContext}, render_asset::RenderAssets, render_graph}};

use crate::{images::GameOfLifeImage, SIM_SIZE, WORKGROUP_SIZE};

#[derive(Resource)]
pub struct GameOfLifePipeline {
    init_pipeline: CachedComputePipelineId,
    update_pipeline: CachedComputePipelineId,
    texture_bind_group_layout: BindGroupLayout,
}

impl FromWorld for GameOfLifePipeline {
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
    
        GameOfLifePipeline {
            texture_bind_group_layout,
            init_pipeline,
            update_pipeline,
        }
    }
}

#[derive(Resource)]
struct GameOfLifeImageBindGroup(BindGroup);

pub fn prepare_bind_group(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    pipeline: Res<GameOfLifePipeline>,
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
    commands.insert_resource(GameOfLifeImageBindGroup(bind_group));
}

pub enum GameOfLifeState{
    Loading,
    Init,
    Update
}

pub struct GameOfLifeNode{
    state: GameOfLifeState
}

impl Default for GameOfLifeNode{
    fn default() -> Self {
        Self { state: GameOfLifeState::Loading }
    }
}

impl render_graph::Node for GameOfLifeNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<GameOfLifePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        // if the corresponding pipeline has loaded, transition to the next stage
        match self.state {
            GameOfLifeState::Loading => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.init_pipeline)
                {
                    self.state = GameOfLifeState::Init;
                }
            }
            GameOfLifeState::Init => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.update_pipeline)
                {
                    self.state = GameOfLifeState::Update;
                }
            }
            GameOfLifeState::Update => {}
        }
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        // dbg!(&world.resource::<GameOfLifeImageBindGroup>().0);
        let texture_bind_group = &world.resource::<GameOfLifeImageBindGroup>().0;
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<GameOfLifePipeline>();

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor{label: Some("cpd")});

        pass.set_bind_group(0, texture_bind_group, &[]);

        // select the pipeline based on the current state
        match self.state {
            GameOfLifeState::Loading | GameOfLifeState::Update=> {}
            GameOfLifeState::Init => {
                let init_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.init_pipeline)
                    .unwrap();
                pass.set_pipeline(init_pipeline);
                pass.dispatch_workgroups(SIM_SIZE.0 / WORKGROUP_SIZE, SIM_SIZE.1 / WORKGROUP_SIZE, 1);
            }
            // GameOfLifeState::Update => {
            //     let update_pipeline = pipeline_cache
            //         .get_compute_pipeline(pipeline.update_pipeline)
            //         .unwrap();
            //     pass.set_pipeline(update_pipeline);
            //     pass.dispatch_workgroups(SIM_SIZE.0 / WORKGROUP_SIZE, SIM_SIZE.1 / WORKGROUP_SIZE, 1);
            // }
        }

        Ok(())
    }
}