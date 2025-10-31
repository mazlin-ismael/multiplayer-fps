use bevy::prelude::*;
use bevy_renet::renet::{transport::NetcodeServerTransport, DefaultChannel, RenetServer, ServerEvent};
use crate::player::{PlayerRegistry, ProjectileRegistry, extract_name_from_user_data};
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
    mut projectiles: ResMut<ProjectileRegistry>,
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
                        if let Some(player_id) = registry.get_player_id_from_temp(client_id_u64) {
                            // Créer le projectile
                            let pos = Vec3::new(position[0], position[1], position[2]);
                            let dir = Vec3::new(direction[0], direction[1], direction[2]);
                            let projectile_id = projectiles.spawn_projectile(player_id, pos, dir);

                            println!("Player {} shot projectile {} from {:?}", player_id, projectile_id, pos);

                            // Informer tous les clients du nouveau projectile
                            let projectile_message = ServerMessage::ProjectileSpawned {
                                projectile_id,
                                shooter_id: player_id,
                                position,
                                direction,
                            };

                            for other_client_id in server.clients_id() {
                                server.send_message(other_client_id, DefaultChannel::ReliableOrdered, projectile_message.to_bytes());
                            }
                        }
                    }
                }
            }
        }
    }
}

// Système pour mettre à jour les projectiles et détecter les collisions
pub fn update_projectiles_system(
    time: Res<Time>,
    mut server: ResMut<RenetServer>,
    mut projectiles: ResMut<ProjectileRegistry>,
    mut players: ResMut<PlayerRegistry>,
) {
    let dt = time.delta_seconds();
    let mut projectiles_to_remove = Vec::new();
    let mut damage_events = Vec::new(); // (projectile_id, hit_player_id, shooter_id)

    // Mettre à jour tous les projectiles
    for (projectile_id, projectile) in projectiles.projectiles.iter_mut() {
        // Déplacer le projectile
        projectile.position += projectile.velocity * dt;
        projectile.lifetime -= dt;

        // Vérifier si le projectile est expiré
        if projectile.lifetime <= 0.0 {
            projectiles_to_remove.push(*projectile_id);
            continue;
        }

        // Vérifier les collisions avec les joueurs
        for (player_id, player_state) in players.players.iter() {
            // Ne pas toucher le tireur
            if *player_id == projectile.shooter_id {
                continue;
            }

            // Hitbox simple: sphère de rayon 1.5m autour du joueur
            let player_pos = Vec3::new(
                player_state.position[0],
                player_state.position[1],
                player_state.position[2],
            );
            let distance = projectile.position.distance(player_pos);

            if distance < 1.5 {
                // Collision détectée!
                println!("Projectile {} hit player {}", projectile_id, player_id);
                damage_events.push((*projectile_id, *player_id, projectile.shooter_id));
                projectiles_to_remove.push(*projectile_id);
                break;
            }
        }
    }

    // Traiter les événements de dommage
    for (projectile_id, hit_player_id, shooter_id) in damage_events {
        // Infliger 1 point de dégâts
        if let Some((new_health, is_dead)) = players.damage_player(hit_player_id, 1) {
            // Informer tous les clients des dégâts
            let damage_message = ServerMessage::PlayerDamaged {
                player_id: hit_player_id,
                new_health,
                attacker_id: shooter_id,
            };

            for client_id in server.clients_id() {
                server.send_message(client_id, DefaultChannel::ReliableOrdered, damage_message.to_bytes());
            }

            // Si le joueur est mort
            if is_dead {
                println!("Player {} was killed by player {}", hit_player_id, shooter_id);

                // Informer tous les clients de la mort
                let death_message = ServerMessage::PlayerDied {
                    player_id: hit_player_id,
                    killer_id: shooter_id,
                };

                for client_id in server.clients_id() {
                    server.send_message(client_id, DefaultChannel::ReliableOrdered, death_message.to_bytes());
                }

                // Le tueur récupère 1 point de vie
                players.heal_player(shooter_id, 1);
                println!("Player {} healed after kill", shooter_id);

                // Respawn le joueur mort après 3 secondes (pour l'instant immédiat)
                // TODO: Ajouter un timer de respawn
                if let Some(spawn_index) = players.get_spawn_index(hit_player_id) {
                    let map = GameMap::from_global().with_spawn_position(spawn_index);
                    let spawn_pos = [map.spawn_x, 1.7, map.spawn_z];
                    players.respawn_player(hit_player_id, spawn_pos);

                    // Informer tous les clients du respawn
                    let respawn_message = ServerMessage::PlayerRespawned {
                        player_id: hit_player_id,
                        position: spawn_pos,
                        health: 3,
                    };

                    for client_id in server.clients_id() {
                        server.send_message(client_id, DefaultChannel::ReliableOrdered, respawn_message.to_bytes());
                    }
                }
            }
        }
    }

    // Supprimer les projectiles qui ont été détruits
    for projectile_id in projectiles_to_remove {
        projectiles.projectiles.remove(&projectile_id);
    }
}