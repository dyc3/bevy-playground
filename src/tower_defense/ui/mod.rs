use bevy::prelude::*;

use super::{waves::WaveStatus, player::Player};

#[derive(Component)]
pub(crate) struct WaveText;

#[derive(Component)]
pub struct PlayerMoneyText;

pub(crate) fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
	commands.spawn_bundle(UiCameraBundle::default());
	commands
		.spawn_bundle(TextBundle {
			style: Style {
				align_self: AlignSelf::FlexEnd,
				..Style::default()
			},
			// Use `Text` directly
			text: Text {
				// Construct a `Vec` of `TextSection`s
				sections: vec![
					TextSection {
						value: "Wave: ".to_string(),
						style: TextStyle {
							font: asset_server.load("fonts/Hack-Regular.ttf"),
							font_size: 60.0,
							color: Color::WHITE,
						},
					},
					TextSection {
						value: "".to_string(),
						style: TextStyle {
							font: asset_server.load("fonts/Hack-Regular.ttf"),
							font_size: 60.0,
							color: Color::WHITE,
						},
					},
				],
				..Text::default()
			},
			..TextBundle::default()
		})
		.insert(WaveText);

	commands
		.spawn_bundle(TextBundle {
			style: Style {
				align_self: AlignSelf::FlexEnd,
				..Style::default()
			},
			// Use `Text` directly
			text: Text {
				// Construct a `Vec` of `TextSection`s
				sections: vec![
					TextSection {
						value: "Money: ".to_string(),
						style: TextStyle {
							font: asset_server.load("fonts/Hack-Regular.ttf"),
							font_size: 40.0,
							color: Color::WHITE,
						},
					},
					TextSection {
						value: "".to_string(),
						style: TextStyle {
							font: asset_server.load("fonts/Hack-Regular.ttf"),
							font_size: 40.0,
							color: Color::WHITE,
						},
					},
				],
				..Text::default()
			},
			..TextBundle::default()
		})
		.insert(PlayerMoneyText);
}

pub(crate) fn update_wave_text(wave_manager: Res<super::waves::WaveManager>, mut query: Query<&mut Text, With<WaveText>>) {
	for mut text in query.iter_mut() {
		// Update the value of the second section
		text.sections[1].value = format!("{} / {}", wave_manager.current_wave_num(), wave_manager.waves.len());
		match wave_manager.wave_status() {
			WaveStatus::Pending => {
				text.sections[1].style.color = Color::WHITE;
			}
			WaveStatus::InProgress => {
				text.sections[1].style.color = Color::GOLD;
			}
			WaveStatus::WaitingForEnemiesToDie => {
				text.sections[1].style.color = Color::ORANGE_RED;
			}
			WaveStatus::Finished => {
				text.sections[1].style.color = Color::GREEN;
			}
		}
	}
}

pub(crate) fn update_money_text(
	player: Query<&Player>,
	mut query: Query<&mut Text, With<PlayerMoneyText>>
) {
	let player = player.single();
	for mut text in query.iter_mut() {
		text.sections[1].value = format!("{}", player.money());
	}
}
