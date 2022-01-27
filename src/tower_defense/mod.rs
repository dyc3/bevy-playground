use bevy::prelude::*;

mod towers;
mod enemy;
mod map;

use crate::tower_defense::enemy::{Wave, WaveManager, WaveStage};

pub struct TowerDefensePlugin;

impl Plugin for TowerDefensePlugin {
	fn build(&self, app: &mut App) {
		app
			.insert_resource(
				WaveManager::new(
					vec![
						Wave {
							stage: WaveStage::new(10, 0.5),
						}
					],
				)
			)
			.add_startup_system(add_camera)
			.add_startup_system(add_lights)
			.add_startup_system(add_path)
			.add_startup_system(add_towers)
			.add_startup_system(add_enemies)
			.add_system(enemy::spawn_enemies_from_waves)
			.add_system(move_enemies)
			.add_system(towers::operate_towers);
	}
}

fn add_camera(mut commands: Commands) {
	commands.spawn_bundle(PerspectiveCameraBundle {
		transform: Transform::from_xyz(0.0, 0.0, 20.0),
		..Default::default()
	});
}

fn add_lights(mut commands: Commands) {
	// ambient light
	commands.insert_resource(AmbientLight {
		color: Color::ORANGE_RED,
		brightness: 0.02,
	});

	// directional 'sun' light
	const HALF_SIZE: f32 = 10.0;
	commands.spawn_bundle(DirectionalLightBundle {
		directional_light: DirectionalLight {
			// Configure the projection to better fit the scene
			shadow_projection: OrthographicProjection {
				left: -HALF_SIZE,
				right: HALF_SIZE,
				bottom: -HALF_SIZE,
				top: HALF_SIZE,
				near: -10.0 * HALF_SIZE,
				far: 10.0 * HALF_SIZE,
				..Default::default()
			},
			shadows_enabled: true,
			..Default::default()
		},
		transform: Transform {
			translation: Vec3::new(0.0, 2.0, 0.0),
			rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
			..Default::default()
		},
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
		Vec3::new(0.0, -5.0, 5.0),
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
	for i in 0..15  {
		commands.spawn_bundle(
			PbrBundle {
				mesh: mesh.clone(),
				material: material.clone(),
				transform: Transform::from_xyz(15.0 - (i * 2) as f32, 3.0, 0.0),
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
