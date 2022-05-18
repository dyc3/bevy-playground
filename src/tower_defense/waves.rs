use bevy::prelude::*;

use crate::tower_defense::enemy::Enemy;

#[derive(Component)]
pub struct WaveManager {
	pub waves: Vec<Wave>,
	pub current_wave_index: usize,
	enemy_spawn_timer: Timer,
	wave_status: WaveStatus,
}

impl WaveManager {
	pub fn new(waves: Vec<Wave>) -> Self {
		Self {
			waves,
			current_wave_index: 0,
			enemy_spawn_timer: Timer::from_seconds(0.5, true),
			wave_status: WaveStatus::Pending,
		}
	}

	pub fn wave_status(&self) -> WaveStatus {
		self.wave_status
	}

	pub fn current_wave_num(&self) -> usize {
		(self.current_wave_index + 1).min(self.waves.len())
	}

	pub fn current_wave(&self) -> &Wave {
		&self.waves[self.current_wave_index]
	}

	pub fn current_wave_mut(&mut self) -> &mut Wave {
		&mut self.waves[self.current_wave_index]
	}

	fn set_wave_status(&mut self, wave_status: WaveStatus) {
		info!("Wave status changed: {:?} => {:?}", self.wave_status(), wave_status);
		self.wave_status = wave_status;
	}
}

#[derive(Debug)]
pub struct Wave {
	pub stage: WaveStage,
}

#[derive(Debug)]
pub struct WaveStage {
	pub enemy_count: u32,
	pub spawn_rate: f32,

	spawned: u32,
}

impl Default for WaveStage {
	fn default() -> Self {
		Self {
			enemy_count: Default::default(),
			spawn_rate: 0.5,

			spawned: 0,
		}
	}
}

impl WaveStage {
	pub fn new(enemy_count: u32, spawn_rate: f32) -> Self {
		Self {
			enemy_count,
			spawn_rate,

			spawned: 0,
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub enum WaveStatus {
	Pending,
	InProgress,
	WaitingForEnemiesToDie,
	Finished,
}

pub fn spawn_enemies_from_waves(
	time: Res<Time>,
	keyboard_input: Res<Input<KeyCode>>,
	mut wave_manager: ResMut<WaveManager>,
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	mut enemies: Query<(Entity, &Enemy), With<Enemy>>,
) {
	if wave_manager.current_wave_index >= wave_manager.waves.len() {
		return;
	}

	match wave_manager.wave_status {
		WaveStatus::Pending => {
			if keyboard_input.pressed(KeyCode::Space) {
				info!("Starting wave {}", wave_manager.current_wave_index);
				wave_manager.enemy_spawn_timer = Timer::from_seconds(
					wave_manager.current_wave().stage.spawn_rate, true
				);
				wave_manager.set_wave_status(WaveStatus::InProgress);
			}
		}
		WaveStatus::InProgress => {
			if wave_manager.enemy_spawn_timer.tick(time.delta()).just_finished() {
				wave_manager.enemy_spawn_timer.reset();
				let wave = wave_manager.current_wave_mut();
				if wave.stage.spawned < wave.stage.enemy_count {
					wave.stage.spawned += 1;
					let enemy = Enemy::new(
						100,
						0,
						0.,
					);
					enemy.spawn(commands, meshes, materials);
				} else {
					wave_manager.set_wave_status(WaveStatus::WaitingForEnemiesToDie);
				}
			}
		}
		WaveStatus::WaitingForEnemiesToDie => {
			if enemies.iter().count() == 0 {
				wave_manager.set_wave_status(WaveStatus::Finished);
			}
		}
		WaveStatus::Finished => {
			info!("Wave complete");
			wave_manager.current_wave_index += 1;
			wave_manager.set_wave_status(WaveStatus::Pending);
		}
	}
}