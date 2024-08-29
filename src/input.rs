use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use bevy::{prelude::*, render::extract_resource::ExtractResource};
use std::time::Duration;

const FRAMES_PER_SECOND: u64 = 1;
#[derive(Debug, Resource, Clone, ExtractResource)]
pub struct AutomataParams {
    pub is_paused: bool,
    pub frame: Arc<AtomicUsize>,
    pub frame_queue_count: Arc<AtomicUsize>,
}

impl Default for AutomataParams {
    fn default() -> Self {
        Self {
            is_paused: false,
            frame: Arc::new(AtomicUsize::new(0)),
            frame_queue_count: Arc::new(AtomicUsize::new(0)),
        }
    }
}

pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AutomataParams>()
            .add_systems(Startup, setup_draw_timer)
            .add_systems(Update, update_input_state)
            .add_systems(FixedUpdate, update_ready);
    }
}

pub fn update_input_state(
    // mut contexts: EguiContexts,
    // window_query: Query<&Window>,
    mut params: ResMut<AutomataParams>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    // camera_q: Query<(&Camera, &GlobalTransform)>,
    // mut mouse_button_input_events: EventReader<MouseButtonInput>,
) {
    // let Ok(primary_window) = window_query.get_single() else {
    //     return;
    // };
    // // get the camera info and transform
    // let Ok((camera, camera_transform)) = camera_q.get_single() else {
    //     return;
    // };

    // let ctx = contexts.ctx_mut();
    // if ctx.wants_pointer_input()
    //     || ctx.is_pointer_over_area()
    //     || ctx.is_using_pointer()
    //     || ctx.wants_pointer_input()
    // {
    //     // GUI gets priority input
    //     params.is_drawing = false;
    //     params.can_scroll = false;
    //     return;
    // } else {
    //     params.can_scroll = true;
    // }

    // Determine button state
    // for event in mouse_button_input_events.iter() {
    //     if event.button == MouseButton::Left {
    //         params.is_drawing = event.state == ButtonState::Pressed;
    //     }
    // }

    // Pause the simulation
    if keyboard_input.just_pressed(KeyCode::Space) {
        params.is_paused = !params.is_paused;
    }

    if keyboard_input.just_pressed(KeyCode::KeyF) {
        params.frame_queue_count.store(1, Ordering::SeqCst); // need to store 2* number of frames to draw (2 frames = 1 cycle)
    }

    // if let Some(world_position) = primary_window
    //     .cursor_position()
    //     .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
    //     .map(|ray| ray.origin.truncate())
    // {
    //     params.prev_mouse_pos = params.mouse_pos;
    //     params.mouse_pos =
    //         crate::utils::world_pos_to_canvas_pos(world_position * Vec2::new(1.0, -1.0));
    // }
}

#[derive(Resource)]
pub struct DrawTimer {
    timer: Timer,
}

pub fn update_ready(mut timer: ResMut<DrawTimer>, params: ResMut<AutomataParams>, time: Res<Time>) {
    if params.is_paused {
        return;
    }
    timer.timer.tick(time.delta());
    if timer.timer.just_finished() {
        eprintln!("Frame {}", params.frame.load(Ordering::SeqCst));
        params.frame_queue_count.store(1, Ordering::SeqCst);
    }
}

fn setup_draw_timer(mut commands: Commands) {
    commands.insert_resource(DrawTimer {
        timer: Timer::new(
            Duration::from_secs(1 / FRAMES_PER_SECOND),
            TimerMode::Repeating,
        ),
    })
}
