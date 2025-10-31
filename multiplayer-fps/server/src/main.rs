mod network;
mod player;
mod systems;

use bevy::prelude::*;
use bevy::MinimalPlugins;
use bevy_renet::{RenetServerPlugin, transport::NetcodeServerPlugin};

use network::create_network_resources;
use player::{PlayerRegistry, ProjectileRegistry};
use systems::*;

fn main() {
    let (server, transport) = create_network_resources();
    let game_map = shared::GameMap::from_global();

    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(RenetServerPlugin)
        .add_plugins(NetcodeServerPlugin)
        .insert_resource(server)
        .insert_resource(transport)
        .insert_resource(PlayerRegistry::default())
        .insert_resource(ProjectileRegistry::default())
        .insert_resource(game_map)
        .add_systems(Update, handle_connection_events)
        .add_systems(Update, handle_player_messages) // NOUVEAU
        .add_systems(Update, update_projectiles_system) // Système de projectiles
        .run();
}