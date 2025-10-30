mod network;
mod player;
mod systems;

use bevy::prelude::*;
use bevy::MinimalPlugins;
use bevy_renet::{RenetServerPlugin, transport::NetcodeServerPlugin};

use network::create_network_resources;
use player::PlayerRegistry;
use systems::*;

fn main() {
    let (server, transport) = create_network_resources();

    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(RenetServerPlugin)
        .add_plugins(NetcodeServerPlugin)
        .insert_resource(server)
        .insert_resource(transport)
        .insert_resource(PlayerRegistry::default())
        .add_systems(Update, handle_connection_events)
        .add_systems(Update, handle_player_messages) // NOUVEAU
        .run();
}