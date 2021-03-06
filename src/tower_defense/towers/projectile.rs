use std::cmp::Ordering;

use bevy::prelude::*;

use crate::{tower_defense::{enemy::Enemy, map::Path}, pid_controller::PidControlled};

#[derive(Component, Debug)]
pub struct TowerProjectile {
	pub damage: u32,
	pub speed: f32,
	pub target: Entity,

	last_pos: Option<Vec3>,
}

impl TowerProjectile {
	pub fn new(damage: u32, speed: f32, target: Entity) -> Self {
		Self {
			damage,
			speed,
			target,
			last_pos: Default::default()
		}
	}

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
	paths: Query<&Path>,
	mut projectiles: Query<(&mut TowerProjectile, &mut Transform)>,
	objects: Query<(&Transform, &Enemy), Without<TowerProjectile>>,
) {
	for (mut projectile, mut transform) in projectiles.iter_mut() {
		let result = objects.get(projectile.target);
		if result.is_err() {
			continue;
		}
		let (target, enemy) = result.unwrap();
		// The target object's current position
		let target_pos = target.translation;
		// The approximate time in the future that the projectile will hit the target
		let collision_time_delta = transform.translation.distance(target_pos) / projectile.speed;
		let path = paths.iter()
			.find(|path| path.id == enemy.path_id)
			.expect(format!("No path with id: {}", enemy.path_id).as_str());
		// The position to aim for. Here, we predict where the target will be in the future.
		let predicted_pos = path.get_point_along_path(enemy.path_pos + enemy.speed * collision_time_delta);
		let mut objective_pos = predicted_pos;

		// HACK: if the projectile really close to the predicted position, it probably means
		// that the projectile is jittering in place.
		// Make a beeline for the actual target's position instead. Idk if this is a good idea.
		if predicted_pos.distance(transform.translation) < 0.1 {
			objective_pos = target_pos;
		}

		let current_velocity = if let Some(last_pos) = projectile.last_pos {
			last_pos - transform.translation
		} else {
			Vec3::new(0.0, 0.0, 0.0)
		};
		let desired_velocity = (objective_pos - transform.translation).normalize() * projectile.speed;
		let steering = desired_velocity - current_velocity;

		let new_velocity = current_velocity + steering;
		let new_pos = transform.translation + new_velocity * time.delta().as_secs_f32();
		transform.translation = new_pos;

		projectile.last_pos = Some(transform.translation);
	}
}

pub fn projectile_collisions(
	mut commands: Commands,
	mut projectiles: Query<(Entity, &TowerProjectile, &Transform)>,
	mut enemies: Query<(Entity, &mut Enemy, &Transform), Without<TowerProjectile>>,
) {
	for (entity, mut projectile, transform) in projectiles.iter_mut() {
		// check the projectile is close enough to ANY enemy
		for (enemy_entity, mut enemy, enemy_transform) in enemies.iter_mut() {
			if transform.translation.distance(enemy_transform.translation) < 0.5 {
				commands.entity(entity).despawn();
				enemy.hurt(projectile.damage);
				break;
			}
		}
	}
}

pub fn retarget_projectiles(
	mut commands: Commands,
	mut projectiles: Query<(Entity, &mut TowerProjectile, &Transform)>,
	mut enemies: Query<(Entity, &Transform), (With<Enemy>, Without<TowerProjectile>)>,
) {
	for (entity, mut projectile, transform) in projectiles.iter_mut() {
		let result = enemies.get_mut(projectile.target);
		if result.is_err() {
			// retarget if there are more enemies
			if enemies.iter().count() > 0 {
				projectile.target = enemies.iter().min_by(|x, y| {
					let x_dist = (x.1.translation - transform.translation).length();
					let y_dist = (y.1.translation - transform.translation).length();
					x_dist.partial_cmp(&y_dist).unwrap_or_else(|| Ordering::Equal)
				}).unwrap().0;
			} else {
				commands.entity(entity).despawn();
			}
		}
	}
}
