use bevy::prelude::*;
use bevy::ecs::event::Events;

mod player;
mod towers;
mod enemy;
mod exp_level;
mod map;
mod ui;
mod waves;

use crate::camera::{self, PanOrbitCamera};
use crate::tower_defense::waves::{Wave, WaveManager, WaveStage};
use crate::pid_controller::{self, PidControlled};

use self::enemy::{EnemyCreateOptions, EventEnemyDeath};
use self::exp_level::{ExperienceBus, ExpLevel};
use player::Player;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, SystemLabel)]
enum SimulationStepLabel {
	/// Core game logic. Eg. Movement, collision, etc.
	Logic,
	Visual,
	/// Reward the player for accomplishments.
	Reward,
	/// Clean up, prepare for next frame.
	Cleanup,
}

pub struct TowerDefensePlugin;

impl Plugin for TowerDefensePlugin {
	fn build(&self, app: &mut App) {
		let expbus = ExperienceBus::new();
		let deathbus: Events<EventEnemyDeath> = Events::default();

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
							stage: WaveStage::new(20, 0.20, EnemyCreateOptions {
								health: 100,
								speed: 10.,
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
			.insert_resource(expbus)
			.insert_resource(deathbus)
			.add_startup_system(add_camera)
			.add_startup_system(add_lights)
			.add_startup_system(add_path)
			.add_startup_system(add_towers)
			.add_startup_system(add_player)
			.add_startup_system(ui::setup_ui)
			.add_system(pid_controller::system_pid_controller_position)
			.add_system(waves::spawn_enemies_from_waves)
			.add_system_set(
				SystemSet::new()
					.label(SimulationStepLabel::Logic)
					.before(SimulationStepLabel::Reward)
					.with_system(enemy::move_enemies)
					.with_system(enemy::monitor_health)
			)
			// .add_system(enemy::move_enemies_with_pid)
			.add_system(towers::operate_towers)
			.add_system(towers::tower_smooth_look.label(SimulationStepLabel::Visual))
			.add_system_set(
				SystemSet::new()
					.label(SimulationStepLabel::Visual)
					.with_system(towers::laser::aim_lasers)
					.with_system(towers::laser::update_laser_locks)
					.with_system(towers::laser::clean_up_expired_lasers)
					.with_system(map::visualize_path)
			)
			.add_system(towers::projectile::move_projectiles)
			.add_system(towers::projectile::projectile_collisions)
			.add_system(towers::projectile::retarget_projectiles)
			.add_system_set(
				SystemSet::new()
					.label(SimulationStepLabel::Reward)
					.with_system(enemy::process_enemy_death)
					.with_system(towers::handle_tower_level_up)
					.with_system(exp_level::process_experience_gain)
					.with_system(exp_level::process_level_ups)
			)
			.add_system_set(
				SystemSet::new()
					.label(SimulationStepLabel::Cleanup)
					.after(SimulationStepLabel::Reward)
					.with_system(exp_level::update_exp_bus)
					.with_system(Events::<EventEnemyDeath>::update_system)
			)
			.add_system(ui::update_wave_text)
			.add_system(ui::update_money_text)
			.add_system(camera::pan_orbit_camera);
	}
}

fn add_camera(mut commands: Commands) {
	commands.spawn_bundle(PerspectiveCameraBundle {
		transform: Transform::from_xyz(0.0, 0.0, 20.0),
		..Default::default()
	}).insert(PanOrbitCamera {
		radius: 20.0,
		..Default::default()
	});
}

fn add_lights(mut commands: Commands) {
	// ambient light
	commands.insert_resource(AmbientLight {
		color: Color::WHITE,
		brightness: 0.15,
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
	let visualizers = 5;
	for i in 0..path.points().len() - 1 {
		for j in 0..visualizers {
			let mesh = meshes.add(Mesh::from(shape::UVSphere {
				radius: 0.05,
				sectors: 10,
				stacks: 10,
			}));
			let material = materials.add(StandardMaterial {
				base_color: Color::WHITE,
				metallic: 1.,
				perceptual_roughness: 0.5,
				..Default::default()
			});
			commands.spawn_bundle(
				PbrBundle {
					mesh: mesh,
					material: material,
					..Default::default()
				}
			).insert(map::PathVisualizer {
				path_id: path.id,
				node_start: i,
				node_end: i + 1,
				offset: j as f32 / visualizers as f32,
			});
		}
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
			.insert(tower)
			.insert(ExpLevel::new());
	}
}

fn add_player(
	mut commands: Commands,
) {
	commands.spawn()
		.insert(Player::new())
		.insert(ExpLevel::new());
}
