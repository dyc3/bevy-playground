use bevy::prelude::*;

use crate::{tower_defense::enemy, pid_controller::PidControlled};

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
	mut enemy: Query<(&mut enemy::Enemy, &Transform, Entity), Without<Tower>>,
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	for (mut tower, mut transform) in towers.iter_mut() {
		let mut enemies_in_range = enemy.iter_mut().filter(|e| transform.translation.distance(e.1.translation) < tower.range).collect::<Vec<_>>();
		let target_enemy = match tower.targeting {
			TowerTargeting::First => {
				enemies_in_range.iter_mut().min_by(|a, b| {
					b.0.path_pos.partial_cmp(&a.0.path_pos).unwrap()
				})
			},
			TowerTargeting::Closest => {
				enemies_in_range.iter_mut().min_by(|a, b| {
					transform.translation.distance(a.1.translation).partial_cmp(&transform.translation.distance(b.1.translation)).unwrap()
				})
			},
		};

		if let Some((enemy, enemy_pos, enemy_entity)) = target_enemy {
			// make the tower look at the closest enemy
			transform.look_at(enemy_pos.translation, Vec3::new(0.0, 1.0, 0.0));

			// tower attacks
			if tower.attack_timer.tick(time.delta()).just_finished() {
				enemy.hurt(10);
				let proj = TowerProjectile {
					damage: 10,
					speed: 10.,
					target: *enemy_entity,
				};
				proj.spawn(transform.translation, &mut commands, &mut meshes, &mut materials);
				tower.attack_timer.reset();
			}
		}
	}
}

#[derive(Component, Debug)]
pub struct TowerProjectile {
	pub damage: u32,
	pub speed: f32,
	pub target: Entity,
}

impl TowerProjectile {
	pub fn spawn(
		self,
		pos: Vec3,
		commands: &mut Commands,
		meshes: &mut ResMut<Assets<Mesh>>,
		materials: &mut ResMut<Assets<StandardMaterial>>,
	) {
		let mesh = meshes.add(Mesh::from(shape::Icosphere { radius: 0.1, subdivisions: 1 }));
		let material = materials.add(StandardMaterial {
			base_color: Color::BLUE,
			..Default::default()
		});
		commands.spawn_bundle(
			PbrBundle {
				mesh: mesh,
				material: material,
				transform: Transform::from_xyz(pos.x, pos.y, pos.z),
				..Default::default()
			}
		)
			.insert(self);
	}
}

pub fn move_projectiles(
	time: Res<Time>,
	mut projectiles: Query<(&TowerProjectile, &mut Transform)>,
	objects: Query<&Transform, Without<TowerProjectile>>,
) {
	for (projectile, mut transform) in projectiles.iter_mut() {
		let result = objects.get(projectile.target);
		if result.is_err() {
			continue;
		}
		let target = result.unwrap();
		info!("MOVE PROJECTILE");

		let up = Vec3::new(0.0, 1.0, 0.0);
		let forward = Vec3::normalize(transform.translation - target.translation);
		let right = up.cross(forward).normalize();
		let up = forward.cross(right);
		let look_at = Quat::from_mat3(&Mat3::from_cols(right, up, forward));

		transform.rotation = transform.rotation.slerp(look_at, 2. * time.delta().as_secs_f32());
		let move_delta = transform.forward() * projectile.speed * time.delta().as_secs_f32();
		transform.translation += move_delta;
	}
}
