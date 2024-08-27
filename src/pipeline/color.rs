use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_graph::{self, RenderLabel},
        render_resource::*,
        renderer::*,
        texture::GpuImage,
        Render, RenderSet,
    },
};
use std::{borrow::Cow, sync::atomic::Ordering};

use super::automata::{GameOfLifeBuffers, GameOfLifeImage, GameOfLifeImageBindGroup};
use crate::{input::AutomataParams, NUM_OF_CELLS, SIZE, WORKGROUP_SIZE};

pub struct AutomataColorPipelinePlugin;
impl Plugin for AutomataColorPipelinePlugin {
    fn build(&self, render_app: &mut App) {
        render_app
            .init_resource::<AutomataColorPipeline>()
            .add_systems(
                Render,
                prepare_color_bind_group.in_set(RenderSet::PrepareBindGroups),
            );
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct AutomataColorLabel;

#[derive(Resource)]
pub struct AutomataColorPipeline {
    color_pipeline: CachedComputePipelineId,
    color_bind_group_layout: BindGroupLayout,
}

impl FromWorld for AutomataColorPipeline {
    fn from_world(world: &mut World) -> Self {
        let pipeline_cache = world.resource::<PipelineCache>();

        let color_bind_group_layout = world.resource::<RenderDevice>().create_bind_group_layout(
            Some("Game of Life Color Bind Group Layout"),
            &[
                BindGroupLayoutEntry {
                    binding: 0,
                    count: None,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new((2 * std::mem::size_of::<u32>()) as _),
                    },
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    count: None,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(
                            (NUM_OF_CELLS
                                * (std::mem::size_of::<u32>()            // alive: u32
                                    + 4 * (std::mem::size_of::<f32>()))) // color: vec4<f32>
                                as _,
                        ),
                    },
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    count: None,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(
                            (NUM_OF_CELLS
                                * (std::mem::size_of::<u32>()            // alive: u32
                                    + 4 * (std::mem::size_of::<f32>()))) // color: vec4<f32>
                                as _,
                        ),
                    },
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadWrite,
                        format: TextureFormat::Rgba8Unorm,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        );

        let color_shader = world.resource::<AssetServer>().load("shaders/color.wgsl");

        let color_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            shader: color_shader,
            shader_defs: vec![],
            push_constant_ranges: vec![],
            entry_point: Cow::from("color_cells"),
            layout: vec![color_bind_group_layout.clone()],
            label: Some(std::borrow::Cow::Borrowed("Game of Life Color Pipeline")),
        });

        AutomataColorPipeline {
            color_pipeline,
            color_bind_group_layout,
        }
    }
}

// ================================== BindGroup ================================== //

#[derive(Resource)]
struct AutomataColorBindGroups(pub BindGroup);

pub fn prepare_color_bind_group(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    buffers: Res<GameOfLifeBuffers>,
    params: Res<AutomataParams>,
    pipeline: Res<AutomataColorPipeline>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    game_of_life_image: Res<GameOfLifeImage>,
) {
    let frame = params.frame.load(Ordering::SeqCst);
    let (buffer_in, buffer_out) = if frame % 2 == 0 {
        (&buffers.in_out[0], &buffers.in_out[1])
    } else {
        (&buffers.in_out[1], &buffers.in_out[0])
    };
    let view = gpu_images.get(&game_of_life_image.texture).unwrap();
    let color_bind_group = render_device.create_bind_group(
        Some("Game of Life Color Bind Group"),
        &pipeline.color_bind_group_layout,
        &BindGroupEntries::sequential((
            buffers.size.as_entire_binding(),
            buffer_in.as_entire_binding(),
            buffer_out.as_entire_binding(),
            &view.texture_view,
        )),
    );
    commands.insert_resource(AutomataColorBindGroups(color_bind_group));
}

// ================================== Nodes ================================== //
pub enum AutomataColorState {
    Loading,
    Update,
}

pub struct AutomataColorNode {
    state: AutomataColorState,
}

impl Default for AutomataColorNode {
    fn default() -> Self {
        Self {
            state: AutomataColorState::Loading,
        }
    }
}

impl render_graph::Node for AutomataColorNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<AutomataColorPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        // if the corresponding pipeline has loaded, transition to the next stage
        match self.state {
            AutomataColorState::Loading => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.color_pipeline)
                {
                    self.state = AutomataColorState::Update;
                }
            }
            AutomataColorState::Update => {}
        }
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let bind_group = &world.resource::<GameOfLifeImageBindGroup>().0;
        let color_bind_group = &world.resource::<AutomataColorBindGroups>().0;
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<AutomataColorPipeline>();

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        pass.set_bind_group(0, bind_group, &[]);

        // select the pipeline based on the current state
        match self.state {
            AutomataColorState::Loading => {}
            AutomataColorState::Update => {
                let color_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.color_pipeline)
                    .unwrap();

                pass.set_pipeline(color_pipeline);
                pass.set_bind_group(0, color_bind_group, &[]);
                pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);
            }
        }

        Ok(())
    }
}
