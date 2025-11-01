use bevy::prelude::*;
use bevy_renet::renet::{RenetClient, DefaultChannel};
use shared::ServerMessage;
use std::collections::HashMap;
use crate::player_model::{create_player_model, TankTurret};

// Component pour identifier un autre joueur
#[derive(Component)]
pub struct OtherPlayer {
    pub player_id: u64,
    pub name: String,
    pub health: u8,
    pub score: u32,
}

#[allow(dead_code)]
impl OtherPlayer {
    pub fn player_id(&self) -> u64 {
        self.player_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

// Component pour l'effet de dommage (flash rouge)
#[derive(Component)]
pub struct DamageFlash {
    pub timer: Timer,
}

// Resource pour tracker les autres joueurs
#[derive(Resource, Default)]
pub struct OtherPlayers {
    pub players: HashMap<u64, Entity>,
}

// Resource pour tracker les scores de tous les joueurs (incluant local)
#[derive(Resource, Default)]
pub struct PlayerScores {
    pub scores: HashMap<u64, (String, u32)>, // player_id -> (name, score)
    pub local_player_id: Option<u64>,
}

// Système pour recevoir les messages du serveur sur les autres joueurs
pub fn receive_other_players_system(
    mut client: ResMut<RenetClient>,
    mut commands: Commands,
    mut other_players: ResMut<OtherPlayers>,
    mut player_scores: ResMut<PlayerScores>,
    mut local_health: ResMut<crate::ui_hud::LocalPlayerHealth>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut player_query: Query<&mut OtherPlayer>,
    children_query: Query<&Children>,
    mut transform_query: Query<&mut Transform>,
    turret_query: Query<Entity, With<TankTurret>>,
    // Ajout pour traiter aussi les messages MapData
    mut current_map: ResMut<crate::network::CurrentMap>,
    mut state: ResMut<crate::network::ConnectionState>,
) {
    while let Some(message) = client.receive_message(DefaultChannel::ReliableOrdered) {
        if let Some(server_msg) = ServerMessage::from_bytes(&message) {
            match server_msg {
                // Traiter MapData ici aussi pour éviter de perdre des messages
                ServerMessage::MapData { data } => {
                    if !state.map_received {
                        println!("DEBUG: Received map data {} bytes", data.len());
                        if let Some(map) = shared::GameMap::from_bytes(&data) {
                            println!("Map received from server!");
                            map.display();
                            current_map.0 = Some(map);
                            state.map_received = true;
                        } else {
                            println!("ERROR: Failed to parse map from bytes");
                        }
                    }
                }

                ServerMessage::PlayerJoined { player_id, name, position, health, score } => {
                    println!("Player {} ({}) joined at {:?} with {} health and {} score", player_id, name, position, health, score);

                    // Stocker le score du joueur
                    player_scores.scores.insert(player_id, (name.clone(), score));

                    // Créer la représentation visuelle de l'autre joueur avec un modèle 3D procédural
                    let player_model = create_player_model(&mut commands, &mut meshes, &mut materials);

                    // Attacher le modèle avec la position et le component OtherPlayer
                    commands.entity(player_model).insert((
                        Transform::from_xyz(position[0], position[1], position[2]),
                        OtherPlayer {
                            player_id,
                            name: name.clone(),
                            health,
                            score,
                        },
                    ));

                    other_players.players.insert(player_id, player_model);
                }
                
                ServerMessage::PlayerUpdate { player_id, position, rotation } => {
                    // Mettre à jour la position et rotation du tank
                    if let Some(&tank_entity) = other_players.players.get(&player_id) {
                        // rotation[0] = yaw (rotation horizontale)
                        // rotation[1] = pitch (rotation verticale)
                        // Ajouter PI (180°) au yaw pour inverser car le canon était à l'envers
                        let yaw = rotation[0] + std::f32::consts::PI;
                        let pitch = rotation[1]; // Déjà limité à ±30° côté client

                        // Mettre à jour la position ET la rotation du tank entier (yaw)
                        if let Ok(mut tank_transform) = transform_query.get_mut(tank_entity) {
                            tank_transform.translation = Vec3::new(position[0], position[1], position[2]);
                            // Le tank entier tourne avec le yaw (châssis + tourelle)
                            tank_transform.rotation = Quat::from_rotation_y(yaw);
                        }

                        // Fonction récursive pour trouver une entité avec un marker dans la hiérarchie
                        fn find_entity_recursive<T: Component>(
                            entity: Entity,
                            children_query: &Query<&Children>,
                            target_query: &Query<Entity, With<T>>,
                        ) -> Option<Entity> {
                            // Vérifier si cette entité a le marker
                            if target_query.get(entity).is_ok() {
                                return Some(entity);
                            }
                            // Sinon, chercher dans les enfants
                            if let Ok(children) = children_query.get(entity) {
                                for &child in children.iter() {
                                    if let Some(found) = find_entity_recursive(child, children_query, target_query) {
                                        return Some(found);
                                    }
                                }
                            }
                            None
                        }

                        // Trouver la tourelle et appliquer seulement le pitch (elle hérite du yaw du parent)
                        if let Some(turret_entity) = find_entity_recursive(tank_entity, &children_query, &turret_query) {
                            if let Ok(mut turret_transform) = transform_query.get_mut(turret_entity) {
                                // Appliquer SEULEMENT le pitch à la tourelle (rotation relative au tank)
                                // Le yaw est hérité du parent (tank_entity)
                                // INVERSER le pitch car sinon c'est à l'envers
                                turret_transform.rotation = Quat::from_rotation_x(-pitch);
                            }
                        }
                    }
                }
                
                ServerMessage::PlayerLeft { player_id } => {
                    println!("Player {} left", player_id);

                    // Supprimer le joueur de la scène
                    if let Some(entity) = other_players.players.remove(&player_id) {
                        commands.entity(entity).despawn_recursive();
                    }

                    // Supprimer le score
                    player_scores.scores.remove(&player_id);
                }

                ServerMessage::PlayerDamaged { player_id, new_health, attacker_id } => {
                    println!("Player {} damaged by {} - new health: {}", player_id, attacker_id, new_health);

                    // Vérifier si c'est nous ou un autre joueur
                    if let Some(&entity) = other_players.players.get(&player_id) {
                        // C'est un autre joueur
                        // Mettre à jour la santé
                        if let Ok(mut other_player) = player_query.get_mut(entity) {
                            other_player.health = new_health;
                        }

                        // Ajouter le composant de flash de dommage
                        commands.entity(entity).insert(DamageFlash {
                            timer: Timer::from_seconds(0.3, TimerMode::Once),
                        });
                    } else {
                        // C'est le joueur local qui a été touché
                        // Définir le player_id local si pas encore fait
                        if player_scores.local_player_id.is_none() {
                            player_scores.local_player_id = Some(player_id);
                            // Initialiser le score du joueur local
                            player_scores.scores.insert(player_id, ("You".to_string(), 0));
                        }

                        // Mettre à jour la santé locale
                        local_health.health = new_health;
                        println!("LOCAL PLAYER damaged! New health: {}", new_health);
                    }
                }

                ServerMessage::PlayerDied { player_id, killer_id } => {
                    println!("Player {} was killed by {}", player_id, killer_id);

                    // On pourrait ajouter une animation de mort ici
                    // Pour l'instant on garde le tank visible
                }

                ServerMessage::PlayerRespawned { player_id, position, health } => {
                    println!("Player {} respawned at {:?} with {} health", player_id, position, health);

                    // Vérifier si c'est nous ou un autre joueur
                    if let Some(&entity) = other_players.players.get(&player_id) {
                        // C'est un autre joueur
                        if let Ok(mut tank_transform) = transform_query.get_mut(entity) {
                            tank_transform.translation = Vec3::new(position[0], position[1], position[2]);
                        }
                        if let Ok(mut other_player) = player_query.get_mut(entity) {
                            other_player.health = health;
                        }

                        // Retirer le flash de dommage s'il existe
                        commands.entity(entity).remove::<DamageFlash>();
                    } else {
                        // C'est le joueur local qui respawn
                        // Définir le player_id local si pas encore fait
                        if player_scores.local_player_id.is_none() {
                            player_scores.local_player_id = Some(player_id);
                            // Initialiser le score du joueur local
                            player_scores.scores.insert(player_id, ("You".to_string(), 0));
                        }

                        // Mettre à jour la santé locale
                        local_health.health = health;
                        println!("LOCAL PLAYER respawned with {} health!", health);
                    }
                }

                ServerMessage::ScoreUpdate { player_id, new_score } => {
                    println!("Player {} score updated to {}", player_id, new_score);

                    // Mettre à jour le score dans la resource
                    if let Some((name, score)) = player_scores.scores.get_mut(&player_id) {
                        *score = new_score;
                    }

                    // Mettre à jour le score dans le component si c'est un autre joueur
                    if let Some(&entity) = other_players.players.get(&player_id) {
                        if let Ok(mut other_player) = player_query.get_mut(entity) {
                            other_player.score = new_score;
                        }
                    }
                }

                ServerMessage::MapData { .. } => {
                    // Déjà géré dans receive_map_system
                }
            }
        }
    }
}

// Système pour gérer l'effet de flash rouge quand un tank est touché
pub fn damage_flash_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut DamageFlash, &Children)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    material_query: Query<&Handle<StandardMaterial>>,
    children_query: Query<&Children>,
) {
    for (entity, mut flash, children) in query.iter_mut() {
        flash.timer.tick(time.delta());

        // Calculer l'intensité du flash (1.0 au début, 0.0 à la fin)
        let flash_intensity = 1.0 - flash.timer.fraction();

        // Fonction récursive pour parcourir tous les enfants
        fn apply_flash_recursive(
            entity: Entity,
            flash_intensity: f32,
            materials: &mut ResMut<Assets<StandardMaterial>>,
            material_query: &Query<&Handle<StandardMaterial>>,
            children_query: &Query<&Children>,
        ) {
            // Appliquer le flash sur cette entité si elle a un matériau
            if let Ok(material_handle) = material_query.get(entity) {
                if let Some(material) = materials.get_mut(material_handle) {
                    // Interpoler vers le rouge selon l'intensité du flash
                    let red_tint = flash_intensity;
                    material.base_color = Color::srgb(
                        0.2 + red_tint * 0.8,  // Rouge: de 0.2 à 1.0
                        0.3 * (1.0 - red_tint), // Vert: de 0.3 à 0
                        0.2 * (1.0 - red_tint), // Bleu: de 0.2 à 0
                    );
                }
            }

            // Appliquer récursivement sur tous les enfants
            if let Ok(children) = children_query.get(entity) {
                for &child in children.iter() {
                    apply_flash_recursive(child, flash_intensity, materials, material_query, children_query);
                }
            }
        }

        // Appliquer le flash à tous les enfants récursivement
        for &child in children.iter() {
            apply_flash_recursive(child, flash_intensity, &mut materials, &material_query, &children_query);
        }

        // Supprimer le composant quand le flash est terminé
        if flash.timer.finished() {
            // Remettre les couleurs originales sur tous les enfants récursivement
            fn reset_colors_recursive(
                entity: Entity,
                materials: &mut ResMut<Assets<StandardMaterial>>,
                material_query: &Query<&Handle<StandardMaterial>>,
                children_query: &Query<&Children>,
            ) {
                if let Ok(material_handle) = material_query.get(entity) {
                    if let Some(material) = materials.get_mut(material_handle) {
                        // Couleur verte militaire originale pour le tank
                        // (on remet la couleur par défaut, les différentes parties auront leur couleur)
                        material.base_color = Color::srgb(0.2, 0.3, 0.2);
                    }
                }

                if let Ok(children) = children_query.get(entity) {
                    for &child in children.iter() {
                        reset_colors_recursive(child, materials, material_query, children_query);
                    }
                }
            }

            for &child in children.iter() {
                reset_colors_recursive(child, &mut materials, &material_query, &children_query);
            }

            commands.entity(entity).remove::<DamageFlash>();
        }
    }
}