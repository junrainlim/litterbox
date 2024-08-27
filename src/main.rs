//! A compute shader that simulates Conway's Game of Life.
//!
//! Compute shaders use the GPU for computing arbitrary information, that may be independent of what
//! is rendered to the screen.

use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};
use iyes_perf_ui::prelude::*;
use litterbox::GameOfLifeComputePlugin;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: ((600) as f32, (800) as f32).into(),
                        // uncomment for unthrottled FPS
                        // present_mode: bevy::window::PresentMode::AutoNoVsync,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            FrameTimeDiagnosticsPlugin,
            PerfUiPlugin,
            GameOfLifeComputePlugin,
        ))
        .run();
}
