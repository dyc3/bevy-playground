use bevy::prelude::*;

mod towers;
mod enemy;
mod map;
mod ui;
mod waves;

use crate::tower_defense::waves::{Wave, WaveManager, WaveStage};
use crate::pid_controller::{self, PidControlled};

use self::enemy::EnemyCreateOptions;

pub struct TowerDefensePlugin;

impl Plugin for TowerDefensePlugin {
	fn build(&self, app: &mut App) {
		app
			.insert_resource(
				WaveManager::new(
					vec![
						Wave {
							stage: WaveStage::new(5, 0.25, EnemyCreateOptions {
								health: 50,
								speed: 5.,
								path_id: 0,
							}),
						},
						Wave {
							stage: WaveStage::new(10, 0.5, EnemyCreateOptions {
								health: 100,
								speed: 3.,
								path_id: 0,
							}),
						},
						Wave {
							stage: WaveStage::new(1000, 0.1, EnemyCreateOptions {
								health: 30,
								speed: 0.25,
								path_id: 0,
							}),
						},
					],
				)
			)
			.add_startup_system(add_camera)
			.add_startup_system(add_lights)
			.add_startup_system(add_path)
			.add_startup_system(add_towers)
			.add_startup_system(ui::setup_ui)
			.add_system(pid_controller::system_pid_controller_position)
			.add_system(waves::spawn_enemies_from_waves)
			.add_system(enemy::move_enemies)
			// .add_system(enemy::move_enemies_with_pid)
			.add_system(enemy::monitor_health)
			.add_system(towers::operate_towers)
			.add_system(towers::tower_smooth_look)
			.add_system(towers::move_projectiles)
			.add_system(towers::projectile_collisions)
			.add_system(ui::update_wave_text);
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
	let path = map::Path::new(
		0,
		vec![
		Vec3::new(-15.0, 0.0, 0.0),
		Vec3::new(-5.0, 0.0, 0.0),
		Vec3::new(-5.0, 0.0, 4.0),
		Vec3::new(-5.0, -5.0, 5.0),
		Vec3::new(0.0, -5.0, 5.0),
		Vec3::new(0.0, 5.0, 5.0),
		Vec3::new(5.0, 5.0, 5.0),
		Vec3::new(5.0, 5.0, 0.0),
		Vec3::new(0.0, 5.0, 0.0),
		Vec3::new(-5.0, 2.0, 5.0),
		Vec3::new(5.0, 0.0, 0.0),
		Vec3::new(20.0, 0.0, 0.0),
	]);
	let mesh = meshes.add(Mesh::from(shape::UVSphere {
		radius: 0.2,
		sectors: 20,
		stacks: 20,
	}));
	let material = materials.add(StandardMaterial {
		base_color: Color::WHITE,
		metallic: 1.,
		perceptual_roughness: 0.5,
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

const PID_CONTROL_LOOK_AT: u64 = 1;

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

	let mut tower_positions = vec![
		Vec3::from((-15., -5., 0.)),
		Vec3::from((-10., -5., 0.)),
	];

	for i in 0..15  {
		tower_positions.push(Vec3::from((15.0 - (i * 2) as f32, 3.0, 0.0)));
	}

	for pos in tower_positions {
		let mut tower = towers::Tower::new();
		tower.attack_type = towers::TowerAttackType::Laser;
		commands.spawn_bundle(
			PbrBundle {
				mesh: mesh.clone(),
				material: material.clone(),
				transform: Transform::from_xyz(pos.x, pos.y, pos.z),
				..Default::default()
			}
		)
			.insert(PidControlled::<Vec3, PID_CONTROL_LOOK_AT>::new(0.1, 0., 0.01))
			.insert(tower);
	}
}
