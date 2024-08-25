use bevy::prelude::*;
use bevy_life::ConwayCellState;

pub fn color_states(mut query: Query<(&mut Sprite, &ConwayCellState), Changed<ConwayCellState>>) {
    for (mut sprite, state) in query.iter_mut() {
        let new_color = match state {
            ConwayCellState(false) => Color::srgb(0., 0., 0.),
            ConwayCellState(true) => Color::srgb(2., 2., 2.),
        };
        sprite.color = new_color;
    }
}
