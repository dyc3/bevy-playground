use std::time::Duration;

use bevy::prelude::*;

use crate::{tower_defense::enemy, pid_controller::PidControlled};

use self::{laser::{TowerLaser, TowerLaserLock}, projectile::TowerProjectile};

use super::exp_level::{ExpLevel, ExperienceBus, EventExpGain};

pub mod laser;
pub mod projectile;

#[derive(Component)]
pub struct Tower {
	pub range: f32,
	/// Rate of fire in units per second.
	pub base_attack_rate: f32,
	attack_rate: f32,
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
			base_attack_rate: 1.,
			attack_timer: Timer::from_seconds(1.0, true),
			targeting: TowerTargeting::default(),
			attack_type: TowerAttackType::default(),
			..Default::default()
		}
	}

	/// Rate of fire in units per second.
	pub fn attack_rate(&self, level: u64) -> f32 {
		self.base_attack_rate + (level as f32 * 0.1)
	}

	/// Recalculate Tower stats based on level.
	pub fn update_stats(&mut self, level: u64) {
		self.attack_rate = self.attack_rate(level);
		self.attack_timer.set_duration(Duration::from_secs_f32(1.0 / self.attack_rate));
	}
}

impl Default for Tower {
	fn default() -> Self {
		Self {
			range: 10.,
			base_attack_rate: 1.,
			attack_rate: 1.,
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
	mut expbus: ResMut<ExperienceBus>,
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
			for _ in 0..tower.attack_timer.tick(time.delta()).times_finished() {
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
						let proj = TowerProjectile::new(15, 10., *enemy_entity);
						proj.spawn(transform.translation, transform.rotation, &mut commands, &mut meshes, &mut materials);
					},
				}
				expbus.experience_gain.send(EventExpGain{ entity: tower_entity, experience: 1 });
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

pub fn handle_tower_level_up(
	expbus: Res<ExperienceBus>,
	mut towers: Query<(&mut Tower, &ExpLevel), Changed<ExpLevel>>,
) {
	let mut reader = expbus.level_up.get_reader();
	for event in reader.iter(&expbus.level_up) {
		let result = towers.get_mut(event.entity);
		if let Ok((mut tower, tower_level)) = result {
			tower.update_stats(tower_level.level());
		}
	}
}
