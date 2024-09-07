use bevy::{
    prelude::*,
    render::{
        extract_resource::ExtractResource,
        render_graph::{self, RenderLabel},
        render_resource::*,
        renderer::*,
        Render, RenderSet,
    },
};
use std::{borrow::Cow, sync::atomic::Ordering};

use crate::{AutomataParams, NUM_OF_CELLS, SIZE, WORKGROUP_SIZE};

const SHADER_ASSET_PATH: &str = "shaders/litterbox.wgsl";

pub const BIND_GROUP_LAYOUT_ENTRY_CELL: BindGroupLayoutEntry = BindGroupLayoutEntry {
    binding: u32::MAX,
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
};

pub struct AutomataPipelinePlugin;
impl Plugin for AutomataPipelinePlugin {
    fn build(&self, render_app: &mut App) {
        render_app
            .init_resource::<GameOfLifePipeline>()
            .add_systems(
                Render,
                prepare_automata_bind_group.in_set(RenderSet::PrepareBindGroups),
            );
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct GameOfLifeLabel;

#[derive(Resource, Clone, ExtractResource)]
pub struct GameOfLifeImage {
    pub texture: Handle<Image>,
}

#[derive(Resource, Clone, ExtractResource)]
pub struct GameOfLifeBuffers {
    pub size: Buffer,
    pub in_out: Vec<Buffer>,
}

#[derive(Resource)]
pub struct GameOfLifePipeline {
    texture_bind_group_layout: BindGroupLayout,
    init_pipeline: CachedComputePipelineId,
    update_pipeline: CachedComputePipelineId,
}

impl FromWorld for GameOfLifePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let texture_bind_group_layout = render_device.create_bind_group_layout(
            "GameOfLifeImages Bind Group Layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    BindGroupLayoutEntry {
                        binding: u32::MAX, // is ignored, using value to suppress warning
                        count: None,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: BufferSize::new(
                                (2 * std::mem::size_of::<u32>()) as _,
                            ),
                        },
                    },
                    BIND_GROUP_LAYOUT_ENTRY_CELL,
                    BIND_GROUP_LAYOUT_ENTRY_CELL,
                ),
            ),
        );
        let shader = world.load_asset(SHADER_ASSET_PATH);
        let pipeline_cache = world.resource::<PipelineCache>();
        let init_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![texture_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("init"),
        });
        let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![texture_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("update"),
        });

        GameOfLifePipeline {
            texture_bind_group_layout,
            init_pipeline,
            update_pipeline,
        }
    }
}

#[derive(Resource)]
pub struct GameOfLifeImageBindGroup(pub BindGroup);

pub fn prepare_automata_bind_group(
    mut commands: Commands,
    render_device: Res<RenderDevice>,

    params: Res<AutomataParams>,
    pipeline: Res<GameOfLifePipeline>,
    buffers: Res<GameOfLifeBuffers>,
) {
    // Swap (ping pong) buffers between input and output every frame
    let frame = params.frame.load(Ordering::SeqCst);
    let (buffer_in, buffer_out) = if frame % 2 == 0 {
        (&buffers.in_out[0], &buffers.in_out[1])
    } else {
        (&buffers.in_out[1], &buffers.in_out[0])
    };

    let bind_group = render_device.create_bind_group(
        "Automata Bind Group 0",
        &pipeline.texture_bind_group_layout,
        &BindGroupEntries::sequential((
            buffers.size.as_entire_binding(),
            buffer_in.as_entire_binding(),
            buffer_out.as_entire_binding(),
        )),
    );
    commands.insert_resource(GameOfLifeImageBindGroup(bind_group));
}

enum GameOfLifeState {
    Loading,
    Init,
    Update,
}

pub struct GameOfLifeNode {
    state: GameOfLifeState,
}

impl Default for GameOfLifeNode {
    fn default() -> Self {
        Self {
            state: GameOfLifeState::Loading,
        }
    }
}

impl render_graph::Node for GameOfLifeNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<GameOfLifePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        // if the corresponding pipeline has loaded, transition to the next stage
        match self.state {
            GameOfLifeState::Loading => {
                match pipeline_cache.get_compute_pipeline_state(pipeline.init_pipeline) {
                    CachedPipelineState::Ok(_) => {
                        self.state = GameOfLifeState::Init;
                    }
                    CachedPipelineState::Err(err) => {
                        panic!("Initializing assets/{SHADER_ASSET_PATH}:\n{err}")
                    }
                    _ => {}
                }
            }
            GameOfLifeState::Init => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.update_pipeline)
                {
                    self.state = GameOfLifeState::Update;
                }
            }
            GameOfLifeState::Update => {
                let params = world.resource_mut::<AutomataParams>();
                // if !params.is_paused {
                //     params.steps_left.fetch_add(1, Ordering::SeqCst);
                // }
                if params.steps_left.load(Ordering::SeqCst) > 0 {
                    params.frame.fetch_add(1, Ordering::SeqCst);
                }
            }
        }
    }
    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let params = &world.resource::<AutomataParams>();

        // eprintln!(
        //     "{}, {}",
        //     params.frame.load(Ordering::SeqCst),
        //     params.steps_left.load(Ordering::SeqCst)
        // );

        let is_paused = params.is_paused;

        if is_paused && (params.steps_left.load(Ordering::SeqCst) == 0) {
            return Ok(());
        }

        let automata_bind_group = &world.resource::<GameOfLifeImageBindGroup>().0;
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<GameOfLifePipeline>();

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        pass.set_bind_group(0, automata_bind_group, &[]);

        // select the pipeline based on the current state
        match self.state {
            GameOfLifeState::Loading => {}
            GameOfLifeState::Init => {
                let init_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.init_pipeline)
                    .unwrap();

                pass.set_pipeline(init_pipeline);
                pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);
            }
            GameOfLifeState::Update => {
                let update_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.update_pipeline)
                    .unwrap();
                pass.set_pipeline(update_pipeline);
                pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);
            }
        }

        if params.steps_left.load(Ordering::SeqCst) > 0 {
            params.steps_left.fetch_sub(1, Ordering::SeqCst);
        }
        Ok(())
    }
}
