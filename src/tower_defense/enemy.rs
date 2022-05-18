use bevy::prelude::*;

use crate::tower_defense::map;

#[derive(Debug, Clone, Copy)]
pub struct EnemyCreateOptions {
	pub health: u32,
	pub speed: f32,
	pub path_id: u64,
}

#[derive(Component, Debug)]
pub struct Enemy {
	pub health: u32,
	pub max_health: u32,
	pub path_id: u64,
	pub path_pos: f32,
	/// Speed that the enemy travels in units per second.
	pub speed: f32,
}

impl Enemy {
	pub fn new(options: EnemyCreateOptions) -> Self {
		Self {
			health: options.health,
			max_health: options.health,
			path_id: options.path_id,
			path_pos: 0.,
			speed: options.speed,
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
	let mut enemy = Enemy::new(EnemyCreateOptions {
		health: 10,
		speed: 1.,
		path_id: 0,
	});
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
		enemy.path_pos += enemy.speed * time.delta().as_secs_f32();
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
		if enemy.health <= 0 {
			commands.entity(entity).despawn();
			continue;
		}
		let mat = materials.get_mut(material_handle).expect("no material found");
		// FIXME: changes color for all enemies that share this material instead of just this one. maybe I have to do some shader stuff?
		mat.base_color = Color::from(Vec4::from(Color::WHITE).lerp(Vec4::from(Color::RED), enemy.health_percent()));
	}
}
