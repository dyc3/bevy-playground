use bevy::prelude::*;

use crate::tower_defense::map;

#[derive(Component, Debug)]
pub struct Enemy {
	pub health: u32,
	pub max_health: u32,
	pub path_id: u64,
	pub path_pos: f32,
}

impl Enemy {
	pub fn new(max_health: u32, path_id: u64, path_pos: f32) -> Enemy {
		Enemy {
			health: max_health,
			max_health,
			path_id,
			path_pos,
		}
	}

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

	/// Spawns the enemy in the world.
	pub fn spawn(
		self,
		mut commands: Commands,
		mut meshes: ResMut<Assets<Mesh>>,
		mut materials: ResMut<Assets<StandardMaterial>>,
	) {
		let mesh = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
		let material = materials.add(StandardMaterial {
			base_color: Color::PINK,
			..Default::default()
		});
		commands.spawn_bundle(
			PbrBundle {
				mesh: mesh.clone(),
				material: material.clone(),
				transform: Transform::from_xyz(-10000., 10000., 0.),
				..Default::default()
			}
		)
			.insert(self);
	}

	pub fn health_percent(&self) -> f32 {
		self.health as f32 / self.max_health as f32
	}
}

#[test]
fn test_enemy_hurt() {
	// TODO: move to doctest for hurt
	let mut enemy = Enemy::new(10, 0, 0.);
	enemy.hurt(5);
	assert_eq!(enemy.health, 5);
	enemy.hurt(20);
	assert_eq!(enemy.health, 0);
}

pub(crate) fn move_enemies(
	time: Res<Time>,
	mut query: Query<(&mut Enemy, &mut Transform), With<Enemy>>,
	path: Query<&map::Path>,
) {
	for enemy in query.iter_mut() {
		let (mut enemy, mut transform) = enemy;
		let path = path.iter()
			.find(|path| path.id == enemy.path_id)
			.expect(format!("No path with id: {}", enemy.path_id).as_str());
		enemy.path_pos += time.delta().as_secs_f32() * 0.1;
		transform.translation = path.get_point_along_path(enemy.path_pos);
	}
}

pub(crate) fn monitor_health(
	mut commands: Commands,
	mut query: Query<(Entity, &Enemy, &Handle<StandardMaterial>), With<Enemy>>,
	mut materials: ResMut<Assets<StandardMaterial>>
) {
	for enemy in query.iter_mut() {
		let (entity, enemy, material_handle) = enemy;
		let mat = materials.get_mut(material_handle).expect("no material found");
		// FIXME: changes color for all enemies that share this material instead of just this one. maybe I have to do some shader stuff?
		mat.base_color = Color::from(Vec4::from(Color::WHITE).lerp(Vec4::from(Color::RED), enemy.health_percent()));
		if enemy.health <= 0 {
			commands.entity(entity).despawn();
		}
	}
}
