use bevy::prelude::*;

#[derive(Component)]
pub struct Tower {
	pub range: f32,
	pub attack_timer: Timer,
	pub targeting: TowerTargeting,
}

#[derive(Debug, Clone, Copy)]
pub enum TowerTargeting {
	First,
	Closest,
}

impl Default for TowerTargeting {
    fn default() -> Self {
        TowerTargeting::First
    }
}
