use bevy::{prelude::*, ecs::event::Events};

/// Defines what level this entity is, and manages the experience required to level up.
#[derive(Component, Default)]
pub struct ExpLevel {
	experience: u64,
	level: u64,
}

impl ExpLevel {
	pub fn new() -> Self { Default::default() }

	pub fn experience(&self) -> u64 {
		self.experience
	}

	/// Add experience. Check [`need_level_up()`] for whether the entity is ready to level up.
	pub fn add_experience(&mut self, experience: u64) {
		self.experience += experience;
	}

	pub fn level(&self) -> u64 {
		self.level
	}

	/// Get what the tower's level should be based on the earned experience.
	/// If this value is different than the current level, the tower is ready
	/// to level up.
	fn level_from_exp(&self) -> u64 {
		(self.experience as f64).log2().ceil() as u64 // TODO: fine tune this formula, copilot made it
	}

	pub fn need_level_up(&self) -> bool {
		self.level_from_exp() != self.level
	}

	pub fn apply_level_up(&mut self) {
		self.level = self.level_from_exp();
	}
}

pub fn process_level_ups(
	mut expbus: ResMut<ExperienceBus>,
	mut objects: Query<(Entity, &mut ExpLevel), Changed<ExpLevel>>,
) {
	for (entity, mut obj) in objects.iter_mut() {
		if obj.need_level_up() {
			obj.apply_level_up();
			debug!("Object leveled up to {}.", obj.level());
			expbus.level_up.send(EventLevelUp { entity });
		}
	}
}

#[derive(Component, Debug)]
pub struct ExperienceBus {
	pub experience_gain: Events<EventExpGain>,
	pub level_up: Events<EventLevelUp>,
}

impl ExperienceBus {
	pub fn new() -> Self {
		Self {
			experience_gain: Events::default(),
			level_up: Events::default(),
		}
	}

	pub fn update(&mut self) {
		self.experience_gain.update();
		self.level_up.update();
	}
}

pub fn update_exp_bus(
	mut expbus: ResMut<ExperienceBus>,
) {
	expbus.update();
}

pub fn process_experience_gain(
	expbus: Res<ExperienceBus>,
	mut objs: Query<&mut ExpLevel>,
) {
	let mut reader = expbus.experience_gain.get_reader();
	for event in reader.iter(&expbus.experience_gain) {
		let result = objs.get_mut(event.entity);
		if let Ok(mut level) = result {
			level.add_experience(event.experience);
		} else {
			error!("Entity {:?} does not have an ExpLevel component.", event.entity);
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventExpGain {
	pub entity: Entity,
	pub experience: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventLevelUp {
	pub entity: Entity,
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_add_experience() {
		let mut explevel = ExpLevel {
			experience: 0,
			level: 0,
		};
		explevel.add_experience(10);
		assert_eq!(explevel.experience(), 10);
		explevel.add_experience(10);
		assert_eq!(explevel.experience(), 20);
	}

	#[test]
	fn test_needs_level_up() {
		let explevel = ExpLevel {
			experience: 100000,
			level: 0,
		};
		assert!(explevel.need_level_up());
	}

	#[test]
	fn test_apply_level_up() {
		let mut explevel = ExpLevel {
			experience: 1000,
			level: 0,
		};
		assert!(explevel.level_from_exp() > 0);
		explevel.apply_level_up();
		assert_eq!(explevel.level(), explevel.level_from_exp());
	}
}
