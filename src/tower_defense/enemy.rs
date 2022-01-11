use bevy::prelude::*;

#[derive(Component)]
pub struct Enemy {
	pub health: u32,
	pub path_pos: f32,
}

impl Enemy {
	/// Hurt the enemy for the given amount of damage.
	///
	/// ```
	/// let mut enemy = Enemy { health: 10 };
	/// enemy.hurt(5);
	/// assert_eq!(enemy.health, 5);
	/// enemy.hurt(20);
	/// assert_eq!(enemy.health, 0);
	/// ```
	pub fn hurt(&mut self, damage: u32) {
		if damage > self.health {
			self.health = 0;
		} else {
			self.health -= damage;
		}
	}
}

#[test]
fn test_enemy_hurt() {
	// TODO: move to doctest for hurt
	let mut enemy = Enemy { health: 10, path_pos: 0.0 };
	enemy.hurt(5);
	assert_eq!(enemy.health, 5);
	enemy.hurt(20);
	assert_eq!(enemy.health, 0);
}