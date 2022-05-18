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
	Finished,
}

pub fn spawn_enemies_from_waves(
	time: Res<Time>,
	keyboard_input: Res<Input<KeyCode>>,
	mut wave_manager: ResMut<WaveManager>,
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	if wave_manager.current_wave_index >= wave_manager.waves.len() {
		return;
	}

	match wave_manager.wave_status {
		WaveStatus::Pending => {
			if keyboard_input.pressed(KeyCode::Space) {
				println!("Starting wave {}", wave_manager.current_wave_index);
				wave_manager.wave_status = WaveStatus::InProgress;
				wave_manager.enemy_spawn_timer = Timer::from_seconds(
					wave_manager.current_wave().stage.spawn_rate, true
				);
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
					// TODO: check to see if all enemies are dead
					wave_manager.wave_status = WaveStatus::Finished;
				}
			}
		}
		WaveStatus::Finished => {
			println!("Wave complete");
			wave_manager.current_wave_index += 1;
			wave_manager.wave_status = WaveStatus::Pending;
		}
	}
}