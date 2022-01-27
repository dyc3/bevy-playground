use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct Enemy {
	pub health: u32,
	pub path_pos: f32,
}

impl Enemy {
	/// Hurt the enemy for the given amount of damage.
	///
	/// ```
	/// let mut enemy = Enemy { health: 10 };
	/// enemy.hurt(5);
	/// assert_eq!(enemy.health, 5);
	/// enemy.hurt(20);
	/// assert_eq!(enemy.health, 0);
	/// ```
	pub fn hurt(&mut self, damage: u32) {
		if damage > self.health {
			self.health = 0;
		} else {
			self.health -= damage;
		}
	}

	/// Spawns the enemy in the world.
	pub fn spawn(
		self,
		mut commands: Commands,
		mut meshes: ResMut<Assets<Mesh>>,
		mut materials: ResMut<Assets<StandardMaterial>>,
	) {
		let mesh = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
		let material = materials.add(StandardMaterial {
			base_color: Color::PINK,
			..Default::default()
		});
		commands.spawn_bundle(
			PbrBundle {
				mesh: mesh.clone(),
				material: material.clone(),
				transform: Transform::from_xyz(0., 0., 0.),
				..Default::default()
			}
		)
			.insert(self);
	}
}

#[test]
fn test_enemy_hurt() {
	// TODO: move to doctest for hurt
	let mut enemy = Enemy { health: 10, path_pos: 0.0 };
	enemy.hurt(5);
	assert_eq!(enemy.health, 5);
	enemy.hurt(20);
	assert_eq!(enemy.health, 0);
}

#[derive(Component)]
pub struct WaveManager {
	pub waves: Vec<Wave>,
	pub current_wave: usize,
	enemy_spawn_timer: Timer,
	wave_status: WaveStatus,
}

impl WaveManager {
	pub fn new(waves: Vec<Wave>) -> Self {
		Self {
			waves,
			current_wave: 0,
			enemy_spawn_timer: Timer::from_seconds(0.5, true),
			wave_status: WaveStatus::Pending,
		}
	}

	pub fn current_wave(&self) -> &Wave {
		&self.waves[self.current_wave]
	}

	pub fn current_wave_mut(&mut self) -> &mut Wave {
		&mut self.waves[self.current_wave]
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
	if wave_manager.current_wave >= wave_manager.waves.len() {
		return;
	}

	match wave_manager.wave_status {
		WaveStatus::Pending => {
			if keyboard_input.pressed(KeyCode::Space) {
				println!("Starting wave {}", wave_manager.current_wave);
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
					let enemy = Enemy {
						health: 100,
						path_pos: 0.
					};
					enemy.spawn(commands, meshes, materials);
				} else {
					wave_manager.wave_status = WaveStatus::Finished;
				}
			}
		}
		WaveStatus::Finished => {
			wave_manager.current_wave += 1;
			wave_manager.wave_status = WaveStatus::Pending;
		}
	}
}
