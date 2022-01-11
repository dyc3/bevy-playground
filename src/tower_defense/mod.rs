use bevy::prelude::*;

mod towers;
mod enemy;
mod map;

pub struct TowerDefensePlugin;

impl Plugin for TowerDefensePlugin {
	fn build(&self, app: &mut App) {
		app
			.add_startup_system(add_camera)
			.add_startup_system(add_path)
			.add_startup_system(add_towers)
			.add_startup_system(add_enemies)
			.add_system(move_enemies)
			.add_system(operate_towers);
	}
}

fn add_camera(mut commands: Commands) {
	commands.spawn_bundle(PerspectiveCameraBundle {
		transform: Transform::from_xyz(0.0, 0.0, 20.0),
		..Default::default()
	});
}

fn add_path(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	let path = map::Path::new(vec![
		Vec3::new(-15.0, 0.0, 0.0),
		Vec3::new(-5.0, 0.0, 0.0),
		Vec3::new(-5.0, 0.0, 4.0),
		Vec3::new(-5.0, -5.0, 5.0),
		Vec3::new(0.0, 5.0, 5.0),
		Vec3::new(5.0, 5.0, 5.0),
		Vec3::new(5.0, 5.0, 0.0),
		Vec3::new(5.0, 0.0, 0.0),
		Vec3::new(20.0, 0.0, 0.0),
	]);
	let mesh = meshes.add(Mesh::from(shape::Cube { size: 0.25 }));
	let material = materials.add(StandardMaterial {
		base_color: Color::WHITE,
		..Default::default()
	});
	for point in path.points() {
		commands.spawn_bundle(
			PbrBundle {
				mesh: mesh.clone(),
				material: material.clone(),
				transform: Transform::from_translation(point),
				..Default::default()
			}
		);
	}
	commands.spawn()
		.insert(path);
}

fn add_towers(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	let mesh = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
	let material = materials.add(StandardMaterial {
		base_color: Color::WHITE,
		..Default::default()
	});
	for i in 0..10  {
		commands.spawn_bundle(
			PbrBundle {
				mesh: mesh.clone(),
				material: material.clone(),
				transform: Transform::from_xyz(10.0 - (i * 2) as f32, 3.0, 0.0),
				..Default::default()
			}
		)
			.insert(towers::Tower {
				range: 10.0,
				attack_timer: Timer::from_seconds(1., true),
				targeting: towers::TowerTargeting::default(),
			});
	}
}

fn add_enemies(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	let mesh = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
	let material = materials.add(StandardMaterial {
		base_color: Color::PINK,
		..Default::default()
	});
	for i in 0..10  {
		commands.spawn_bundle(
			PbrBundle {
				mesh: mesh.clone(),
				material: material.clone(),
				transform: Transform::from_xyz(-20.0 - (i * 3) as f32, 0.0, 0.0),
				..Default::default()
			}
		)
			.insert(enemy::Enemy {
				health: 100,
				path_pos: - (i as f32 * 0.1),
			});
	}
}

fn move_enemies(
	mut commands: Commands,
	time: Res<Time>,
	mut query: Query<(Entity, &mut enemy::Enemy, &mut Transform), With<enemy::Enemy>>,
	path: Query<&map::Path>,
) {
	let path = path.single();
	for mut enemy in query.iter_mut() {
		enemy.1.path_pos += time.delta().as_secs_f32() * 0.1;
		enemy.2.translation = path.get_point_along_path(enemy.1.path_pos);

		if enemy.1.health <= 0 {
			commands.entity(enemy.0).despawn();
		}
	}
}

fn operate_towers(time: Res<Time>,
	mut towers: Query<(&mut towers::Tower, &mut Transform)>,
	mut enemy: Query<(&mut enemy::Enemy, &Transform), Without<towers::Tower>>,
){
	for mut tower in towers.iter_mut() {
		let mut enemies_in_range = enemy.iter_mut().filter(|e| tower.1.translation.distance(e.1.translation) < tower.0.range).collect::<Vec<_>>();
		let target_enemy = match tower.0.targeting {
			towers::TowerTargeting::First => {
				enemies_in_range.iter_mut().min_by(|a, b| {
					a.0.path_pos.partial_cmp(&b.0.path_pos).unwrap()
				})
			},
			towers::TowerTargeting::Closest => {
				enemies_in_range.iter_mut().min_by(|a, b| {
					tower.1.translation.distance(a.1.translation).partial_cmp(&tower.1.translation.distance(b.1.translation)).unwrap()
				})
			},
		};

		if let Some((enemy, enemy_pos)) = target_enemy {
			// make the tower look at the closest enemy
			if tower.1.translation.distance(enemy_pos.translation) < tower.0.range {
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
}
