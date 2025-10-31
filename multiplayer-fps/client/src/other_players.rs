use bevy::prelude::*;
use bevy_renet::renet::{RenetClient, DefaultChannel};
use shared::ServerMessage;
use std::collections::HashMap;
use crate::player_model::{create_player_model, TankTurret, TankCannon};

// Component pour identifier un autre joueur
#[derive(Component)]
pub struct OtherPlayer {
    pub player_id: u64,
    pub name: String,
}

// Resource pour tracker les autres joueurs
#[derive(Resource, Default)]
pub struct OtherPlayers {
    pub players: HashMap<u64, Entity>,
}

// Système pour recevoir les messages du serveur sur les autres joueurs
pub fn receive_other_players_system(
    mut client: ResMut<RenetClient>,
    mut commands: Commands,
    mut other_players: ResMut<OtherPlayers>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(Entity, &OtherPlayer)>,
    children_query: Query<&Children>,
    mut transform_query: Query<&mut Transform>,
    turret_query: Query<Entity, With<TankTurret>>,
    cannon_query: Query<Entity, With<TankCannon>>,
) {
    while let Some(message) = client.receive_message(DefaultChannel::ReliableOrdered) {
        if let Some(server_msg) = ServerMessage::from_bytes(&message) {
            match server_msg {
                ServerMessage::PlayerJoined { player_id, name, position } => {
                    println!("Player {} ({}) joined at {:?}", player_id, name, position);

                    // Créer la représentation visuelle de l'autre joueur avec un modèle 3D procédural
                    let player_model = create_player_model(&mut commands, &mut meshes, &mut materials);

                    // Attacher le modèle avec la position et le component OtherPlayer
                    commands.entity(player_model).insert((
                        Transform::from_xyz(position[0], position[1], position[2]),
                        OtherPlayer {
                            player_id,
                            name: name.clone(),
                        },
                    ));

                    other_players.players.insert(player_id, player_model);
                }
                
                ServerMessage::PlayerUpdate { player_id, position, rotation } => {
                    // Mettre à jour la position du tank et rotation de la tourelle/canon
                    if let Some(&tank_entity) = other_players.players.get(&player_id) {
                        // rotation[0] = yaw (rotation horizontale de la tourelle)
                        // rotation[1] = pitch (rotation verticale du canon)
                        // Ajouter PI (180°) au yaw pour inverser car le canon était à l'envers
                        let yaw = rotation[0] + std::f32::consts::PI;
                        let pitch = rotation[1];

                        // Limiter le pitch du canon entre -30° et +30° (environ -0.52 et +0.52 radians)
                        const MAX_PITCH: f32 = 0.52; // ~30°
                        let clamped_pitch = pitch.clamp(-MAX_PITCH, MAX_PITCH);

                        // Mettre à jour la POSITION du tank (le châssis ne tourne pas)
                        if let Ok(mut tank_transform) = transform_query.get_mut(tank_entity) {
                            tank_transform.translation = Vec3::new(position[0], position[1], position[2]);
                            // PAS de rotation sur le tank entier - seulement la tourelle tourne
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

                        // Trouver la tourelle et appliquer le yaw
                        if let Some(turret_entity) = find_entity_recursive(tank_entity, &children_query, &turret_query) {
                            if let Ok(mut turret_transform) = transform_query.get_mut(turret_entity) {
                                // Appliquer le yaw à la tourelle (rotation autour de Y)
                                turret_transform.rotation = Quat::from_rotation_y(yaw);
                            }

                            // Trouver le canon (enfant de la tourelle) et appliquer le pitch
                            if let Some(cannon_entity) = find_entity_recursive(turret_entity, &children_query, &cannon_query) {
                                if let Ok(mut cannon_transform) = transform_query.get_mut(cannon_entity) {
                                    // Appliquer le pitch au canon (rotation autour de X local)
                                    cannon_transform.rotation = Quat::from_rotation_x(clamped_pitch);
                                }
                            }
                        }
                    }
                }
                
                ServerMessage::PlayerLeft { player_id } => {
                    println!("Player {} left", player_id);
                    
                    // Supprimer le joueur de la scène
                    if let Some(entity) = other_players.players.remove(&player_id) {
                        commands.entity(entity).despawn();
                    }
                }
                
                ServerMessage::MapData { .. } => {
                    // Déjà géré dans receive_map_system
                }
            }
        }
    }
}

// Système pour interpoler les mouvements des autres joueurs (smooth)
pub fn interpolate_other_players_system(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<OtherPlayer>>,
) {
    // Pour l'instant on fait juste un téléport, mais on pourrait faire une interpolation smooth
    // C'est une amélioration future possible
}