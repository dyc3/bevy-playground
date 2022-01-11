use bevy::prelude::*;

mod towers;
mod enemy;

pub struct TowerDefensePlugin;

impl Plugin for TowerDefensePlugin {
	fn build(&self, app: &mut App) {
		app
			.add_startup_system(add_camera)
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
	commands.spawn_bundle(
		PbrBundle {
			mesh: mesh.clone(),
			material: material.clone(),
			transform: Transform::from_xyz(0.0, 3.0, 0.0),
			..Default::default()
		}
	)
		.insert(towers::Tower {
			range: 10.0,
			attack_timer: Timer::from_seconds(0.3, true),
		});
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
			});
	}
}

fn move_enemies(
	mut commands: Commands,
	time: Res<Time>,
	mut query: Query<(Entity, &enemy::Enemy, &mut Transform), With<enemy::Enemy>>,
) {
	for mut enemy in query.iter_mut() {
		enemy.2.translation.x += time.delta().as_secs_f32() * 2.0;

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
		let closest_enemy = enemy.iter_mut().min_by(|a, b| {
			tower.1.translation.distance(a.1.translation).partial_cmp(&tower.1.translation.distance(b.1.translation)).unwrap()
		});
		if let Some((enemy, enemy_pos)) = closest_enemy {
			let mut enemy = enemy; // borrowck workaround

			// make the tower look at the closest enemy
			if tower.1.translation.distance(enemy_pos.translation) < tower.0.range {
				tower.1.look_at(enemy_pos.translation, Vec3::new(0.0, 1.0, 0.0));

				// tower attacks
				if tower.0.attack_timer.tick(time.delta()).just_finished() {
					enemy.health -= 10;
					println!("enemy.health: {}", enemy.health);
					tower.0.attack_timer.reset();
				}
			}

		}
	}
}
