use bevy::prelude::*;
use bevy_renet::renet::{transport::NetcodeServerTransport, DefaultChannel, RenetServer, ServerEvent};
use crate::player::{PlayerRegistry, extract_name_from_user_data};
use shared::{GameMap, ClientMessage, ServerMessage};

pub fn handle_connection_events(
    mut events: EventReader<ServerEvent>,
    mut registry: ResMut<PlayerRegistry>,
    mut commands: Commands,
    mut server: ResMut<RenetServer>,
    transport: Res<NetcodeServerTransport>,
) {
    for event in events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                let client_id_u64 = client_id.raw();
                
                if let Some(user_data) = transport.user_data(*client_id) {
                    let name = extract_name_from_user_data(&user_data);
                    
                    // Ajouter le joueur et obtenir son ID
                    let player_id = registry.add_player(client_id_u64, name.clone(), &mut commands);
                    
                    // Obtenir l'index de spawn pour ce joueur
                    let spawn_index = registry.get_spawn_index(player_id).unwrap_or(0);
                    
                    // Créer une map avec la position de spawn spécifique
                    let map = GameMap::from_global().with_spawn_position(spawn_index);
                    
                    println!("Sending map to client {} (Player ID: {}):", client_id_u64, player_id);
                    map.display();
                    
                    // Envoyer la map via ServerMessage
                    let map_message = ServerMessage::MapData {
                        data: map.to_bytes(),
                    };
                    server.send_message(*client_id, DefaultChannel::ReliableOrdered, map_message.to_bytes());

                    // Informer le nouveau joueur des joueurs déjà présents
                    for (other_id, other_state) in registry.get_all_players_except(player_id) {
                        let join_message = ServerMessage::PlayerJoined {
                            player_id: other_id,
                            name: other_state.name.clone(),
                            position: other_state.position,
                        };
                        server.send_message(*client_id, DefaultChannel::ReliableOrdered, join_message.to_bytes());
                    }

                    // Informer tous les autres joueurs de la connexion du nouveau
                    let new_player_message = ServerMessage::PlayerJoined {
                        player_id,
                        name: name.clone(),
                        position: [map.spawn_x, 1.7, map.spawn_z],
                    };
                    
                    for other_client_id in server.clients_id() {
                        if other_client_id.raw() != client_id_u64 {
                            server.send_message(other_client_id, DefaultChannel::ReliableOrdered, new_player_message.to_bytes());
                        }
                    }
                }
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                let client_id_u64 = client_id.raw();
                println!("Client {} disconnected: {}", client_id_u64, reason);
                
                // Récupérer le player_id avant de supprimer
                if let Some(player_id) = registry.get_player_id_from_temp(client_id_u64) {
                    // Informer tous les clients que ce joueur est parti
                    let left_message = ServerMessage::PlayerLeft { player_id };
                    for other_client_id in server.clients_id() {
                        server.send_message(other_client_id, DefaultChannel::ReliableOrdered, left_message.to_bytes());
                    }
                    
                    registry.remove_player(client_id_u64, &mut commands);
                }
            }
        }
    }
}

// NOUVEAU : Recevoir les mouvements des clients et les broadcaster
pub fn handle_player_messages(
    mut server: ResMut<RenetServer>,
    mut registry: ResMut<PlayerRegistry>,
) {
    // Pour chaque client connecté
    for client_id in server.clients_id() {
        let client_id_u64 = client_id.raw();
        
        // Lire tous les messages de ce client
        while let Some(message) = server.receive_message(client_id, DefaultChannel::ReliableOrdered) {
            if let Some(client_msg) = ClientMessage::from_bytes(&message) {
                match client_msg {
                    ClientMessage::PlayerMovement { position, rotation } => {
                        // Trouver le player_id correspondant
                        if let Some(player_id) = registry.get_player_id_from_temp(client_id_u64) {
                            // Mettre à jour la position du joueur
                            registry.update_player_position(player_id, position, rotation);
                            
                            // Broadcaster à tous les AUTRES clients
                            let update_message = ServerMessage::PlayerUpdate {
                                player_id,
                                position,
                                rotation,
                            };
                            
                            let bytes = update_message.to_bytes();
                            
                            for other_client_id in server.clients_id() {
                                if other_client_id != client_id {
                                    server.send_message(other_client_id, DefaultChannel::ReliableOrdered, bytes.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}