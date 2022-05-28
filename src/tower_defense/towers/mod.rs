use std::time::Duration;

use bevy::prelude::*;

use crate::{tower_defense::enemy, pid_controller::PidControlled};

pub mod projectile;

#[derive(Component)]
pub struct Tower {
	pub range: f32,
	/// Rate of fire in units per second.
	pub base_attack_rate: f32,
	pub attack_timer: Timer,
	pub targeting: TowerTargeting,
	pub attack_type: TowerAttackType,
	experience: u64,
	level: u64,

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
			base_attack_rate: 1.,
			attack_timer: Timer::from_seconds(1.0, true),
			targeting: TowerTargeting::default(),
			attack_type: TowerAttackType::default(),
			..Default::default()
		}
	}

	/// Rate of fire in units per second.
	pub fn attack_rate(&self) -> f32 {
		self.base_attack_rate + (self.level as f32 * 0.1)
	}

	pub fn experience(&self) -> u64 {
		self.experience
	}

	/// Add experience to the tower.
	pub fn add_experience(&mut self, experience: u64) {
		self.experience += experience;
	}

	pub fn level(&self) -> u64 {
		self.level
	}

	/// Get what the tower's level should be based on the earned experience.
	/// If this value is different than the current level, the tower is ready
	/// to level up.
	fn level_from_exp(&self) -> u64 {
		(self.experience as f64).log2().ceil() as u64 // TODO: fine tune this formula, copilot made it
	}

	pub fn need_level_up(&self) -> bool {
		self.level_from_exp() != self.level
	}

	pub fn apply_level_up(&mut self) {
		self.level = self.level_from_exp();
		self.attack_timer.set_duration(Duration::from_secs_f32(1.0 / self.attack_rate()));
	}
}

impl Default for Tower {
	fn default() -> Self {
		Self {
			range: 10.,
			base_attack_rate: 1.,
			attack_timer: Timer::from_seconds(1.0, true),
			targeting: TowerTargeting::default(),
			attack_type: TowerAttackType::default(),
			aim_position: Vec3::new(0., 0., 0.),
			experience: 0,
			level: 0,
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
	mut towers: Query<(&mut Tower, &Transform, &mut PidControlled<Vec3, PID_CONTROL_LOOK_AT>, Entity)>,
	mut enemy: Query<(&mut enemy::Enemy, &Transform, Entity), Without<Tower>>,
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	for (mut tower, transform, mut controller, tower_entity) in towers.iter_mut() {
		let mut enemies_in_range = enemy.iter_mut()
			.filter(|e|
				transform.translation.distance(e.1.translation) < tower.range && e.0.health > 0
			)
			.collect::<Vec<_>>();
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
						let radius = 0.1;
						let mesh = meshes.add(Mesh::from(
							shape::Capsule {
								radius,
								rings: 1,
								depth: 1. - (radius * 2.),
								latitudes: 5,
								longitudes: 5,
								uv_profile: shape::CapsuleUvProfile::Uniform,
							}
						));
						let material = materials.add(StandardMaterial {
							base_color: Color::GREEN,
							..Default::default()
						});
						commands.spawn_bundle(
							PbrBundle {
								mesh,
								material,
								transform: transform.clone(),
								..Default::default()
							}
						)
							.insert(TowerLaser {
								start_pos: transform.translation,
								end_pos: enemy_pos.translation,
								expire_timer: Timer::from_seconds(0.5, false),
								override_expired: false,
							})
							.insert(TowerLaserLock {
								source: tower_entity,
								target: *enemy_entity,
							});
					},
					TowerAttackType::Projectile => {
						let proj = projectile::TowerProjectile {
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

pub fn tower_process_level_ups(
	mut towers: Query<&mut Tower>,
) {
	for mut tower in towers.iter_mut() {
		if tower.need_level_up() {
			tower.apply_level_up();
			info!("Tower leveled up to {}!", tower.level());
		}
	}
}

#[derive(Component, Debug)]
pub struct TowerLaser {
	pub start_pos: Vec3,
	pub end_pos: Vec3,
	pub expire_timer: Timer,
	/// Force the laser to be expired.
	pub override_expired: bool,
}

#[derive(Component, Debug)]
pub struct TowerLaserLock {
	pub source: Entity,
	pub target: Entity,
}

pub fn aim_lasers(
	mut lasers: Query<(&TowerLaser, &mut Transform)>,
) {
	for (laser, mut transform) in lasers.iter_mut() {
		let midpoint = laser.start_pos.lerp(laser.end_pos, 0.5);
		transform.translation = midpoint;
		transform.scale.y = laser.start_pos.distance(laser.end_pos);

		// because the long part of the laser is on the local Y axis
		let current_direction = transform.up();
		// calculate unit vector that is parralel to the line between <start> and <end>
		let new_direction = (laser.end_pos - laser.start_pos).normalize();
		// create a rotation that will rotate <current_direction> to <new_direction>
		let rotation = Quat::from_rotation_arc(current_direction, new_direction);
		transform.rotate(rotation);
	}
}

pub fn update_laser_locks(
	mut lasers: Query<(&TowerLaserLock, &mut TowerLaser)>,
	objects: Query<&Transform, Without<TowerLaser>>,
) {
	for (laser_lock, mut laser) in lasers.iter_mut() {
		if laser_lock.source == laser_lock.target {
			warn!("laser lock source and target are the same");
		}
		let source = objects.get(laser_lock.source);
		let target = objects.get(laser_lock.target);
		if source.is_err() || target.is_err() {
			laser.override_expired = true;
			continue;
		}
		let source = source.unwrap();
		let target = target.unwrap();

		laser.start_pos = source.translation;
		laser.end_pos = target.translation;
	}
}

pub fn clean_up_expired_lasers(
	time: Res<Time>,
	mut commands: Commands,
	mut lasers: Query<(Entity, &mut TowerLaser)>,
) {
	for (entity, mut laser) in lasers.iter_mut() {
		if laser.override_expired || laser.expire_timer.tick(time.delta()).finished() {
			commands.entity(entity).despawn();
		}
	}
}
