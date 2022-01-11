use bevy::prelude::*;

#[derive(Component)]
pub struct Tower {
	pub range: f32,
	pub attack_timer: Timer,
}
