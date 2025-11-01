use bevy::prelude::*;
use bevy_renet::renet::{transport::NetcodeServerTransport, DefaultChannel, RenetServer, ServerEvent};
use crate::player::{PlayerRegistry, extract_name_from_user_data};
use shared::{GameMap, ClientMessage, ServerMessage};

// Resource pour gérer la rotation équitable des spawns
#[derive(Resource)]
pub struct SpawnRotation {
    pub next_index: usize,
    pub last_used_index: Option<usize>,
}

impl Default for SpawnRotation {
    fn default() -> Self {
        Self {
            next_index: 0,
            last_used_index: None,
        }
    }
}

impl SpawnRotation {
    // Obtient le prochain spawn disponible (évite le dernier utilisé si possible)
    pub fn get_next_spawn(&mut self) -> usize {
        let total_spawns = shared::SPAWN_POINTS.len();

        // Si on a plus de 1 spawn et qu'on essaie de réutiliser le dernier
        if total_spawns > 1 && Some(self.next_index) == self.last_used_index {
            // Avancer au suivant
            self.next_index = (self.next_index + 1) % total_spawns;
        }

        let spawn_index = self.next_index;
        self.last_used_index = Some(spawn_index);

        // Préparer le prochain
        self.next_index = (self.next_index + 1) % total_spawns;

        spawn_index
    }
}

pub fn handle_connection_events(
    mut events: EventReader<ServerEvent>,
    mut registry: ResMut<PlayerRegistry>,
    mut commands: Commands,
    mut server: ResMut<RenetServer>,
    transport: Res<NetcodeServerTransport>,
    mut spawn_rotation: ResMut<SpawnRotation>,
) {
    for event in events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                let client_id_u64 = client_id.raw();
                
                if let Some(user_data) = transport.user_data(*client_id) {
                    let name = extract_name_from_user_data(&user_data);
                    
                    // Ajouter le joueur et obtenir son ID
                    let player_id = registry.add_player(client_id_u64, name.clone(), &mut commands);

                    // Obtenir le prochain spawn par rotation
                    let spawn_index = spawn_rotation.get_next_spawn();
                    println!("Player {} spawning at spawn point #{}", player_id, spawn_index + 1);

                    // Créer une map avec le spawn point sélectionné
                    let map = GameMap::from_global().with_spawn_from_rotation(spawn_index);
                    
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
                            score: other_state.score,
                        };
                        server.send_message(*client_id, DefaultChannel::ReliableOrdered, join_message.to_bytes());
                    }

                    // Informer tous les autres joueurs de la connexion du nouveau
                    let new_player_message = ServerMessage::PlayerJoined {
                        player_id,
                        name: name.clone(),
                        position: [map.spawn_x, 1.7, map.spawn_z],
                        health: 3,
                        score: 0,
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
    mut spawn_rotation: ResMut<SpawnRotation>,
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
                                &mut spawn_rotation,
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
    spawn_rotation: &mut ResMut<SpawnRotation>,
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

            // Hitbox du tank - SYSTÈME DE HITBOX COMPOSITE (4 boîtes)
            // Position joueur à y=1.7 (caméra)

            let player_x = player_state.position[0];
            let player_y = player_state.position[1];
            let player_z = player_state.position[2];

            // CHÂSSIS: Cuboid(1.2, 0.4, 1.8) à y=-1.3 relatif
            // Centre réel: y = 1.7 - 1.3 = 0.4
            let chassis_center = Vec3::new(player_x, player_y - 1.3, player_z);
            let chassis_half_width = 0.6;   // 1.2m total
            let chassis_half_height = 0.2;  // 0.4m total
            let chassis_half_depth = 0.9;   // 1.8m total

            let chassis_dx = (current_pos.x - chassis_center.x).abs();
            let chassis_dy = (current_pos.y - chassis_center.y).abs();
            let chassis_dz = (current_pos.z - chassis_center.z).abs();

            let hit_chassis = chassis_dx < chassis_half_width
                           && chassis_dy < chassis_half_height
                           && chassis_dz < chassis_half_depth;

            // TOURELLE: Cuboid(0.8, 0.5, 0.8) à y=-0.9 relatif
            // Centre réel: y = 1.7 - 0.9 = 0.8
            let turret_center = Vec3::new(player_x, player_y - 0.9, player_z);
            let turret_half_width = 0.4;    // 0.8m total
            let turret_half_height = 0.25;  // 0.5m total
            let turret_half_depth = 0.4;    // 0.8m total

            let turret_dx = (current_pos.x - turret_center.x).abs();
            let turret_dy = (current_pos.y - turret_center.y).abs();
            let turret_dz = (current_pos.z - turret_center.z).abs();

            let hit_turret = turret_dx < turret_half_width
                          && turret_dy < turret_half_height
                          && turret_dz < turret_half_depth;

            // CHENILLE GAUCHE: Cuboid(0.15, 0.3, 1.8) à x=-0.65, y=-1.3
            let track_left_center = Vec3::new(player_x - 0.65, player_y - 1.3, player_z);
            let track_half_width = 0.075;   // 0.15m total
            let track_half_height = 0.15;   // 0.3m total
            let track_half_depth = 0.9;     // 1.8m total

            let track_left_dx = (current_pos.x - track_left_center.x).abs();
            let track_left_dy = (current_pos.y - track_left_center.y).abs();
            let track_left_dz = (current_pos.z - track_left_center.z).abs();

            let hit_track_left = track_left_dx < track_half_width
                              && track_left_dy < track_half_height
                              && track_left_dz < track_half_depth;

            // CHENILLE DROITE: Cuboid(0.15, 0.3, 1.8) à x=0.65, y=-1.3
            let track_right_center = Vec3::new(player_x + 0.65, player_y - 1.3, player_z);

            let track_right_dx = (current_pos.x - track_right_center.x).abs();
            let track_right_dy = (current_pos.y - track_right_center.y).abs();
            let track_right_dz = (current_pos.z - track_right_center.z).abs();

            let hit_track_right = track_right_dx < track_half_width
                               && track_right_dy < track_half_height
                               && track_right_dz < track_half_depth;

            // Debug: afficher quand on est à moins de 3m du joueur
            if (chassis_dx < 3.0 && chassis_dz < 3.0) && steps % 20 == 0 {
                println!("  Step {}: Near player {} | ray=[{:.2}, {:.2}, {:.2}]", steps, player_id, current_pos.x, current_pos.y, current_pos.z);
                println!("    Chassis: dx={:.2} dy={:.2} dz={:.2}", chassis_dx, chassis_dy, chassis_dz);
                println!("    Turret: dx={:.2} dy={:.2} dz={:.2}", turret_dx, turret_dy, turret_dz);
                println!("    TrackL: dx={:.2} dy={:.2} dz={:.2}", track_left_dx, track_left_dy, track_left_dz);
                println!("    TrackR: dx={:.2} dy={:.2} dz={:.2}", track_right_dx, track_right_dy, track_right_dz);
            }

            if hit_chassis || hit_turret || hit_track_left || hit_track_right {
                // TOUCHÉ!
                let mut parts = Vec::new();
                if hit_chassis { parts.push("CHASSIS"); }
                if hit_turret { parts.push("TURRET"); }
                if hit_track_left { parts.push("TRACK_L"); }
                if hit_track_right { parts.push("TRACK_R"); }
                let hit_part = parts.join("+");

                println!(">>> HIT! Player {} ({}) at ray={:?}", player_id, hit_part, current_pos);
                if hit_chassis {
                    println!("    Chassis: dx={:.2} dy={:.2} dz={:.2}", chassis_dx, chassis_dy, chassis_dz);
                }
                if hit_turret {
                    println!("    Turret: dx={:.2} dy={:.2} dz={:.2}", turret_dx, turret_dy, turret_dz);
                }
                if hit_track_left {
                    println!("    TrackL: dx={:.2} dy={:.2} dz={:.2}", track_left_dx, track_left_dy, track_left_dz);
                }
                if hit_track_right {
                    println!("    TrackR: dx={:.2} dy={:.2} dz={:.2}", track_right_dx, track_right_dy, track_right_dz);
                }
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

                    // Le tueur récupère 1 point de vie et gagne 1 point (kill)
                    players.heal_player(shooter_id, 1);
                    players.add_kill(shooter_id);

                    // Informer tous les clients du nouveau score
                    if let Some(killer_state) = players.players.get(&shooter_id) {
                        let score_message = ServerMessage::ScoreUpdate {
                            player_id: shooter_id,
                            new_score: killer_state.score,
                        };
                        for client_id in server.clients_id() {
                            server.send_message(client_id, DefaultChannel::ReliableOrdered, score_message.to_bytes());
                        }
                    }

                    // Respawn le joueur mort avec rotation des spawns
                    let spawn_index = spawn_rotation.get_next_spawn();
                    println!("Player {} respawning at spawn point #{}", hit_id, spawn_index + 1);

                    let respawn_map = GameMap::from_global().with_spawn_from_rotation(spawn_index);
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

            return; // Le tir s'arrête au premier joueur touché
        }
    }

    println!("Raycast missed - max distance reached");
}
