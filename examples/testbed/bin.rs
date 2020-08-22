use bevy::prelude::*;

use bevy_prototype_laminar_networking::NetworkingPlugin;

mod game;
mod net;
mod plugin;
mod ui;

fn main() {
    App::build()
        .add_default_plugins()
        .add_plugin(NetworkingPlugin)
        .add_plugin(plugin::TestbedPlugin)
        .run();
}
