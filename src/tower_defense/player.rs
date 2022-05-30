use bevy::prelude::*;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Player {
	money: u64,
}

impl Player {
	pub fn new() -> Self {
		Self { ..Default::default() }
	}

	pub fn money(&self) -> u64 {
		self.money
	}

	pub fn add_money(&mut self, amount: u64) {
		self.money += amount;
	}

	pub fn make_purchase(&mut self, amount: u64) -> Result<(), ()> {
		if self.money >= amount {
			self.money -= amount;
			Ok(())
		} else {
			Err(())
		}
	}
}

mod test {
	use super::*;

	#[test]
	fn test_player_making_purchases() {
		let mut player = Player::new();
		assert_eq!(player.money(), 0);
		player.add_money(10);
		assert_eq!(player.money(), 10);
		assert!(player.make_purchase(10).is_ok());
		assert_eq!(player.money(), 0);
		assert!(player.make_purchase(10).is_err());
		assert_eq!(player.money(), 0);
	}
}
