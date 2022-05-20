use bevy::prelude::*;

use crate::{tower_defense::enemy, pid_controller::PidControlled};

use super::enemy::Enemy;

#[derive(Component)]
pub struct Tower {
	pub range: f32,
	pub attack_timer: Timer,
	pub targeting: TowerTargeting,
	pub attack_type: TowerAttackType,

	/// The position the tower is currently looking at. Used for smoothly turning.
	/// Changing the position that the tower is aiming at should be done through the
	/// corresponding PidControlled component.
	aim_position: Vec3,
}

impl Tower {
	/// TODO: add TowerCreateOptions struct to set parameters
	pub fn new() -> Self {
		Self {
			range: 10.,
			attack_timer: Timer::from_seconds(1.0, true),
			targeting: TowerTargeting::default(),
			attack_type: TowerAttackType::default(),
			aim_position: Vec3::new(0., 0., 0.),
		}
	}
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

#[derive(Debug, Clone, Copy)]
pub enum TowerAttackType {
	Laser,
	Projectile,
}

impl Default for TowerAttackType {
	fn default() -> Self {
		TowerAttackType::Laser
	}
}

const PID_CONTROL_LOOK_AT: u64 = 1;

pub fn operate_towers(
	time: Res<Time>,
	mut towers: Query<(&mut Tower, &Transform, &mut PidControlled<Vec3, PID_CONTROL_LOOK_AT>)>,
	mut enemy: Query<(&mut enemy::Enemy, &Transform, Entity), Without<Tower>>,
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	for (mut tower, transform, mut controller) in towers.iter_mut() {
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
			// transform.look_at(enemy_pos.translation, Vec3::new(0.0, 1.0, 0.0));
			controller.set_target(enemy_pos.translation);

			// tower attacks
			if tower.attack_timer.tick(time.delta()).just_finished() {
				match tower.attack_type {
					TowerAttackType::Laser => {
						enemy.hurt(15);
					},
					TowerAttackType::Projectile => {
						let proj = TowerProjectile {
							damage: 15,
							speed: 10.,
							target: *enemy_entity,
						};
						proj.spawn(transform.translation, transform.rotation, &mut commands, &mut meshes, &mut materials);
					},
				}
				tower.attack_timer.reset();
			}
		}
	}
}

pub fn tower_smooth_look(
	time: Res<Time>,
	mut towers: Query<(&mut Tower, &mut Transform, &mut PidControlled<Vec3, PID_CONTROL_LOOK_AT>)>,
) {
	for (mut tower, mut transform, mut controller) in towers.iter_mut() {
		let aim = tower.aim_position;
		tower.aim_position += controller.compute(time.delta_seconds(), aim);
		transform.look_at(tower.aim_position, Vec3::new(0.0, 1.0, 0.0));
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
		rot: Quat,
		commands: &mut Commands,
		meshes: &mut ResMut<Assets<Mesh>>,
		materials: &mut ResMut<Assets<StandardMaterial>>,
	) {
		let mesh = meshes.add(Mesh::from(shape::Icosphere { radius: 0.1, subdivisions: 1 }));
		let material = materials.add(StandardMaterial {
			base_color: Color::BLUE,
			..Default::default()
		});
		let mut t = Transform::from_xyz(pos.x, pos.y, pos.z);
		t.rotation = rot;
		commands.spawn_bundle(
			PbrBundle {
				mesh: mesh,
				material: material,
				transform: t,
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

		let up = Vec3::new(0.0, 1.0, 0.0);
		let forward = Vec3::normalize(transform.translation - target.translation);
		let right = up.cross(forward).normalize();
		let up = forward.cross(right);
		let look_at = Quat::from_mat3(&Mat3::from_cols(right, up, forward));

		transform.rotation = transform.rotation.slerp(look_at, 4. * time.delta().as_secs_f32());
		let move_delta = transform.forward() * projectile.speed * time.delta().as_secs_f32();
		transform.translation += move_delta;
	}
}

pub fn projectile_collisions(
	mut commands: Commands,
	mut projectiles: Query<(Entity, &mut TowerProjectile, &Transform)>,
	mut enemies: Query<(Entity, &mut Enemy, &Transform), Without<TowerProjectile>>,
) {
	for (entity, mut projectile, transform) in projectiles.iter_mut() {
		let result = enemies.get_mut(projectile.target);
		if result.is_err() {
			// retarget if there are more enemies
			if enemies.iter().count() > 0 {
				projectile.target = enemies.iter().next().unwrap().0;
			} else {
				commands.entity(entity).despawn();
			}
			continue;
		}
		let (enemy_entity, mut enemy, enemy_transform) = result.unwrap();
		if transform.translation.distance(enemy_transform.translation) < 0.5 {
			commands.entity(entity).despawn();
			enemy.hurt(projectile.damage);
		}
	}
}
