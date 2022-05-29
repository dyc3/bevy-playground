use bevy::prelude::*;

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
