mod cell;
mod input;
mod pipeline;
mod utils;

use bevy::{
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    prelude::*,
    render::{
        extract_resource::ExtractResourcePlugin, render_asset::RenderAssetUsages,
        render_graph::RenderGraph, render_resource::*, renderer::RenderDevice, RenderApp,
    },
};
use input::AutomataParams;
use iyes_perf_ui::entries::PerfUiBundle;

use cell::Cell;
use pipeline::{
    automata::{self, GameOfLifeBuffers, GameOfLifeImage, GameOfLifeLabel, GameOfLifeNode},
    color::{self, AutomataColorLabel, AutomataColorNode},
};

const WORKGROUP_SIZE: u32 = 8;

pub const DISPLAY_FACTOR: u32 = 4;
// Should be a multiple of WORKGROUP_SIZE
pub const SIZE: (u32, u32) = (64 * 8 / DISPLAY_FACTOR, 64 * 8 / DISPLAY_FACTOR);
const NUM_OF_CELLS: usize = (SIZE.0 * SIZE.1) as usize;

pub struct GameOfLifeComputePlugin;

impl Plugin for GameOfLifeComputePlugin {
    fn build(&self, app: &mut App) {
        // Extract the game of life image resource from the main world into the render world
        // for operation on by the compute shader and display on the sprite.
        app.add_plugins(ExtractResourcePlugin::<GameOfLifeImage>::default())
            .add_plugins(ExtractResourcePlugin::<GameOfLifeBuffers>::default())
            .add_plugins(ExtractResourcePlugin::<AutomataParams>::default())
            .add_plugins(input::InputPlugin)
            .add_systems(Startup, setup);

        let render_app = app.sub_app_mut(RenderApp);

        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
        render_graph.add_node(GameOfLifeLabel, GameOfLifeNode::default());
        render_graph.add_node(AutomataColorLabel, AutomataColorNode::default());

        render_graph.add_node_edge(GameOfLifeLabel, AutomataColorLabel);
        render_graph.add_node_edge(AutomataColorLabel, bevy::render::graph::CameraDriverLabel);
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .add_plugins(automata::AutomataPipelinePlugin)
            .add_plugins(color::AutomataColorPipelinePlugin);
    }
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>, device: Res<RenderDevice>) {
    let mut image = Image::new_fill(
        Extent3d {
            width: SIZE.0,
            height: SIZE.1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
    let image = images.add(image);

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(SIZE.0 as f32, SIZE.1 as f32)),
            ..default()
        },
        texture: image.clone(),
        transform: Transform::from_scale(Vec3::splat(DISPLAY_FACTOR as f32)),

        ..default()
    });

    let initial_life_data = vec![Cell::default(); NUM_OF_CELLS];
    let buffers_in_out = (0..2)
        .map(|i| {
            utils::create_storage_buffer_with_data(
                &device,
                &initial_life_data,
                Some(&format!("Game of Life Buffer {i}")),
            )
        })
        .collect::<Vec<_>>();

    let buffer_size =
        utils::create_uniform_buffer(&device, &[SIZE.0, SIZE.1], Some("Size Uniform Buffer"));

    commands.insert_resource(GameOfLifeImage { texture: image });
    commands.insert_resource(GameOfLifeBuffers {
        size: buffer_size,
        in_out: buffers_in_out,
    });

    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface,
            ..default()
        },
        BloomSettings::default(),
    ));
    commands.spawn(PerfUiBundle::default());
}
