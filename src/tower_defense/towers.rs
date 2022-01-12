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

pub fn operate_towers(
	time: Res<Time>,
	mut towers: Query<(&mut Tower, &mut Transform)>,
	mut enemy: Query<(&mut enemy::Enemy, &Transform), Without<Tower>>,
) {
	for mut tower in towers.iter_mut() {
		let mut enemies_in_range = enemy.iter_mut().filter(|e| tower.1.translation.distance(e.1.translation) < tower.0.range).collect::<Vec<_>>();
		let target_enemy = match tower.0.targeting {
			TowerTargeting::First => {
				enemies_in_range.iter_mut().min_by(|a, b| {
					b.0.path_pos.partial_cmp(&a.0.path_pos).unwrap()
				})
			},
			TowerTargeting::Closest => {
				enemies_in_range.iter_mut().min_by(|a, b| {
					tower.1.translation.distance(a.1.translation).partial_cmp(&tower.1.translation.distance(b.1.translation)).unwrap()
				})
			},
		};

		if let Some((enemy, enemy_pos)) = target_enemy {
			// make the tower look at the closest enemy
			tower.1.look_at(enemy_pos.translation, Vec3::new(0.0, 1.0, 0.0));

			// tower attacks
			if tower.0.attack_timer.tick(time.delta()).just_finished() {
				enemy.hurt(10);
				println!("enemy.health: {}", enemy.health);
				tower.0.attack_timer.reset();
			}
		}
	}
}
