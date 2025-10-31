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
                            health: other_state.health,
                        };
                        server.send_message(*client_id, DefaultChannel::ReliableOrdered, join_message.to_bytes());
                    }

                    // Informer tous les autres joueurs de la connexion du nouveau
                    let new_player_message = ServerMessage::PlayerJoined {
                        player_id,
                        name: name.clone(),
                        position: [map.spawn_x, 1.7, map.spawn_z],
                        health: 3,
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
    map: Res<GameMap>,
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

                    ClientMessage::Shoot { position, direction } => {
                        // Trouver le player_id correspondant
                        if let Some(shooter_id) = registry.get_player_id_from_temp(client_id_u64) {
                            // RAYCAST instantané
                            let start = Vec3::new(position[0], position[1], position[2]);
                            let dir = Vec3::new(direction[0], direction[1], direction[2]).normalize();

                            println!("\n=== SHOOT DEBUG ===");
                            println!("Player {} shooting from {:?} dir {:?}", shooter_id, start, dir);

                            // Debug: afficher toutes les positions des joueurs
                            for (pid, pstate) in registry.players.iter() {
                                if *pid != shooter_id {
                                    println!("  Target player {} at [{}, {}, {}]",
                                        pid, pstate.position[0], pstate.position[1], pstate.position[2]);
                                }
                            }

                            // Vérifier le raycast contre les murs et les joueurs
                            perform_raycast_hit(
                                shooter_id,
                                start,
                                dir,
                                &mut registry,
                                &map,
                                &mut server,
                            );
                        }
                    }
                }
            }
        }
    }
}

// Fonction pour effectuer un raycast et détecter ce qui est touché
fn perform_raycast_hit(
    shooter_id: u64,
    start: Vec3,
    direction: Vec3,
    players: &mut ResMut<PlayerRegistry>,
    map: &Res<GameMap>,
    server: &mut ResMut<RenetServer>,
) {
    const MAX_RAYCAST_DISTANCE: f32 = 1000.0; // Portée maximale
    const STEP_SIZE: f32 = 0.05; // Précision du raycast augmentée (5cm au lieu de 10cm)

    println!("Starting raycast: start={:?}, dir={:?}", start, direction);

    let mut current_pos = start;
    let step = direction * STEP_SIZE;
    let mut distance_traveled = 0.0;
    let mut steps = 0;

    // Marcher le long du rayon
    while distance_traveled < MAX_RAYCAST_DISTANCE {
        current_pos += step;
        distance_traveled += STEP_SIZE;
        steps += 1;

        // Vérifier collision avec les murs
        let tile_x = current_pos.x.floor() as i32;
        let tile_z = current_pos.z.floor() as i32;

        if tile_x >= 0 && tile_x < map.width as i32 && tile_z >= 0 && tile_z < map.height as i32 {
            let tile = map.tiles[tile_z as usize][tile_x as usize];
            if tile as u8 == 1 { // Mur
                println!("Raycast hit wall at ({}, {})", tile_x, tile_z);
                return; // Le tir s'arrête au mur
            }
        } else {
            // Sorti de la map
            return;
        }

        // Vérifier collision avec les joueurs
        // D'abord trouver quel joueur est touché (sans modifier players)
        let mut hit_player_id: Option<u64> = None;

        for (player_id, player_state) in players.players.iter() {
            // Ne pas toucher le tireur
            if *player_id == shooter_id {
                continue;
            }

            // Hitbox du tank
            let player_pos = Vec3::new(
                player_state.position[0],
                player_state.position[1],
                player_state.position[2],
            );

            // AABB collision - dimensions ajustées pour correspondre au tank visuel
            // Tank visuel: châssis 1.2x0.4x1.8 + tourelle 0.8x0.5x0.8
            let half_width = 0.6;   // Largeur réelle du tank (1.2m total)
            let half_height = 0.5;  // Hauteur approximative du tank (1.0m total)
            let half_depth = 0.9;   // Profondeur du châssis (1.8m total)

            let dx = (current_pos.x - player_pos.x).abs();
            let dy = (current_pos.y - player_pos.y).abs();
            let dz = (current_pos.z - player_pos.z).abs();

            // Debug: afficher quand on est à moins de 3m du joueur
            if dx < 3.0 && dz < 3.0 && steps % 20 == 0 { // Tous les 1m environ
                println!("  Step {}: Near player {} | ray=[{:.2}, {:.2}, {:.2}] player=[{:.2}, {:.2}, {:.2}] | dx={:.2} dy={:.2} dz={:.2}",
                    steps, player_id, current_pos.x, current_pos.y, current_pos.z,
                    player_pos.x, player_pos.y, player_pos.z, dx, dy, dz);
            }

            if dx < half_width && dy < half_height && dz < half_depth {
                // TOUCHÉ!
                println!(">>> HIT! Player {} at ray={:?} player={:?} dx={:.2} dy={:.2} dz={:.2}",
                    player_id, current_pos, player_pos, dx, dy, dz);
                hit_player_id = Some(*player_id);
                break; // Sortir de la boucle
            }
        }

        // Maintenant traiter les dégâts (hors de la boucle pour éviter le borrow checker)
        if let Some(hit_id) = hit_player_id {
            // Infliger 1 point de dégâts
            if let Some((new_health, is_dead)) = players.damage_player(hit_id, 1) {
                // Informer tous les clients des dégâts
                let damage_message = ServerMessage::PlayerDamaged {
                    player_id: hit_id,
                    new_health,
                    attacker_id: shooter_id,
                };

                for client_id in server.clients_id() {
                    server.send_message(client_id, DefaultChannel::ReliableOrdered, damage_message.to_bytes());
                }

                // Si le joueur est mort
                if is_dead {
                    println!("Player {} was killed by player {}", hit_id, shooter_id);

                    // Informer tous les clients de la mort
                    let death_message = ServerMessage::PlayerDied {
                        player_id: hit_id,
                        killer_id: shooter_id,
                    };

                    for client_id in server.clients_id() {
                        server.send_message(client_id, DefaultChannel::ReliableOrdered, death_message.to_bytes());
                    }

                    // Le tueur récupère 1 point de vie
                    players.heal_player(shooter_id, 1);

                    // Respawn le joueur mort
                    if let Some(spawn_index) = players.get_spawn_index(hit_id) {
                        let respawn_map = GameMap::from_global().with_spawn_position(spawn_index);
                        let spawn_pos = [respawn_map.spawn_x, 1.7, respawn_map.spawn_z];
                        players.respawn_player(hit_id, spawn_pos);

                        // Informer tous les clients du respawn
                        let respawn_message = ServerMessage::PlayerRespawned {
                            player_id: hit_id,
                            position: spawn_pos,
                            health: 3,
                        };

                        for client_id in server.clients_id() {
                            server.send_message(client_id, DefaultChannel::ReliableOrdered, respawn_message.to_bytes());
                        }
                    }
                }
            }

            return; // Le tir s'arrête au premier joueur touché
        }
    }

    println!("Raycast missed - max distance reached");
}
