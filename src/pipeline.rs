use std::borrow::Cow;

use bevy::{prelude::*, render::{render_resource::*, renderer::RenderDevice, render_asset::RenderAssets}};

use crate::image::GameOfLifeImage;

#[derive(Resource)]
pub struct GameOfLifePipeline {
    init_pipeline: CachedComputePipelineId,
    update_pipeline: CachedComputePipelineId,
    texture_bind_group_layout: BindGroupLayout,
}

#[derive(Resource)]
struct GameOfLifeImageBindGroup(pub BindGroup);

impl FromWorld for GameOfLifePipeline {
    fn from_world(world: &mut World) -> Self {
        let texture_bind_group_layout =
            world
                .get_resource::<RenderDevice>().unwrap()
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

pub fn queue_bind_group(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    pipeline: Res<GameOfLifePipeline>,
    gpu_images: Res<RenderAssets<Image>>,
    game_of_life_image: Res<GameOfLifeImage>,
) {
    dbg!(&game_of_life_image);
    let view = &gpu_images.get(&game_of_life_image.0).unwrap();

    let bind_group = render_device.create_bind_group(
         Some("Game of Life Bind Group"),
         &pipeline.texture_bind_group_layout,
         &[BindGroupEntry {
            binding: 0,
            resource: BindingResource::TextureView(&view.texture_view),
        }],
    );
    commands.insert_resource(GameOfLifeImageBindGroup(bind_group));
}