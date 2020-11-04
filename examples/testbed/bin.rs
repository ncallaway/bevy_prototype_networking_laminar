use bevy::prelude::*;

use bevy_prototype_networking_laminar::NetworkingPlugin;

mod game;
mod net;
mod plugin;
mod ui;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(NetworkingPlugin)
        .add_plugin(plugin::TestbedPlugin)
        .run();
}
