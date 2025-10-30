use bevy::prelude::*;
use bevy_renet::renet::{RenetClient, DefaultChannel};
use shared::ServerMessage;
use std::collections::HashMap;

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
) {
    while let Some(message) = client.receive_message(DefaultChannel::ReliableOrdered) {
        if let Some(server_msg) = ServerMessage::from_bytes(&message) {
            match server_msg {
                ServerMessage::PlayerJoined { player_id, name, position } => {
                    println!("Player {} ({}) joined at {:?}", player_id, name, position);
                    
                    // Créer la représentation visuelle de l'autre joueur (capsule)
                    let capsule_mesh = meshes.add(Capsule3d::new(0.3, 1.7));
                    let player_material = materials.add(StandardMaterial {
                        base_color: Color::srgb(1.0, 0.3, 0.3), // Rouge pour les autres joueurs
                        ..Default::default()
                    });
                    
                    let entity = commands.spawn((
                        PbrBundle {
                            mesh: capsule_mesh,
                            material: player_material,
                            transform: Transform::from_xyz(position[0], position[1], position[2]),
                            ..Default::default()
                        },
                        OtherPlayer {
                            player_id,
                            name: name.clone(),
                        },
                    )).id();
                    
                    other_players.players.insert(player_id, entity);
                }
                
                ServerMessage::PlayerUpdate { player_id, position, rotation: _ } => {
                    // Mettre à jour la position de l'autre joueur
                    if let Some(&entity) = other_players.players.get(&player_id) {
                        if let Some(mut entity_commands) = commands.get_entity(entity) {
                            entity_commands.insert(Transform::from_xyz(position[0], position[1], position[2]));
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