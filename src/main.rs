use bevy::prelude::*;

mod hello;
mod tower_defense;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // .add_plugin(hello::HelloPlugin)
        .add_plugin(tower_defense::TowerDefensePlugin)
        .run();
}
