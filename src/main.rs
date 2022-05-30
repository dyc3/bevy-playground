use bevy::prelude::*;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

mod hello;
mod tower_defense;
mod pid_controller;
mod camera;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        // .add_plugin(hello::HelloPlugin)
        .add_plugin(tower_defense::TowerDefensePlugin)
        .run();
}
