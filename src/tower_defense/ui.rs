use bevy::prelude::*;

use super::waves::WaveStatus;

#[derive(Component)]
pub(crate) struct WaveText;

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
							color: Color::GOLD,
						},
					},
				],
				..Text::default()
			},
			..TextBundle::default()
		})
		.insert(WaveText);
}

pub(crate) fn update_wave_text(wave_manager: Res<super::waves::WaveManager>, mut query: Query<&mut Text, With<WaveText>>) {
	for mut text in query.iter_mut() {
		// Update the value of the second section
		text.sections[1].value = format!("{} / {}", wave_manager.current_wave_num(), wave_manager.waves.len());
		match wave_manager.wave_status() {
			WaveStatus::Pending => {
				text.sections[1].style.color = Color::GOLD;
			}
			WaveStatus::InProgress => {
				text.sections[1].style.color = Color::WHITE;
			}
			WaveStatus::Finished => {
				text.sections[1].style.color = Color::GREEN;
			}
		}
	}
}
