use bevy::prelude::*;
use crate::other_players::PlayerScores;
use crate::scene::FpsController;

// Component pour marquer l'indicateur de vie
#[derive(Component)]
pub struct HealthIndicator;

// Component pour marquer le scoreboard
#[derive(Component)]
pub struct ScoreboardText;

// Component pour marquer la minimap
#[derive(Component)]
pub struct Minimap;

// Component pour marquer les tuiles de la minimap (générées une fois)
#[derive(Component)]
pub struct MinimapTile;

// Component pour marquer le point du joueur sur la minimap
#[derive(Component)]
pub struct MinimapPlayerDot;

// Resource pour savoir si les tuiles de la minimap ont été générées
#[derive(Resource, Default)]
pub struct MinimapTilesGenerated(pub bool);

// Resource pour tracker la health locale
#[derive(Resource)]
pub struct LocalPlayerHealth {
    pub health: u8,
}

impl Default for LocalPlayerHealth {
    fn default() -> Self {
        Self { health: 3 }
    }
}

// Resource pour savoir si le HUD a été créé
#[derive(Resource, Default)]
pub struct HudSpawned(pub bool);

/// Système pour créer le HUD (health, scoreboard, minimap)
pub fn setup_hud(
    mut commands: Commands,
    mut spawned: ResMut<HudSpawned>,
    cameras: Query<&Camera>,
) {
    // Ne créer qu'une seule fois et seulement si une caméra existe
    if spawned.0 || cameras.is_empty() {
        return;
    }

    spawned.0 = true;

    // === HEALTH INDICATOR (en bas à gauche) ===
    commands.spawn(NodeBundle {
        style: Style {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            bottom: Val::Px(20.0),
            width: Val::Px(200.0),
            height: Val::Px(40.0),
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        background_color: Color::srgba(0.1, 0.1, 0.1, 0.8).into(),
        ..default()
    })
    .with_children(|parent| {
        parent.spawn((
            TextBundle::from_section(
                "❤ ❤ ❤",
                TextStyle {
                    font_size: 24.0,
                    color: Color::srgb(1.0, 0.2, 0.2),
                    ..default()
                },
            ),
            HealthIndicator,
        ));
    });

    // === SCOREBOARD (en haut à gauche) ===
    commands.spawn(NodeBundle {
        style: Style {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(20.0),
            width: Val::Px(250.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        background_color: Color::srgba(0.1, 0.1, 0.1, 0.8).into(),
        ..default()
    })
    .with_children(|parent| {
        parent.spawn((
            TextBundle::from_section(
                "Top: -- (0)\nYou: 0",
                TextStyle {
                    font_size: 18.0,
                    color: Color::srgb(1.0, 1.0, 1.0),
                    ..default()
                },
            ),
            ScoreboardText,
        ));
    });

    // === MINIMAP (en haut à droite) ===
    const MINIMAP_SIZE: f32 = 200.0;
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                right: Val::Px(20.0),
                top: Val::Px(20.0),
                width: Val::Px(MINIMAP_SIZE),
                height: Val::Px(MINIMAP_SIZE),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            background_color: Color::srgba(0.2, 0.2, 0.2, 0.8).into(),
            border_color: Color::srgb(0.5, 0.5, 0.5).into(),
            ..default()
        },
        Minimap,
    ))
    .with_children(|parent| {
        // Point représentant le joueur sur la minimap (centre)
        parent.spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Px(6.0),
                    height: Val::Px(6.0),
                    // Position sera mise à jour dynamiquement
                    ..default()
                },
                background_color: Color::srgb(0.0, 1.0, 0.0).into(), // Vert pour le joueur
                z_index: ZIndex::Global(100), // Au-dessus des tuiles
                ..default()
            },
            MinimapPlayerDot,
        ));
    });
}

/// Système pour mettre à jour l'indicateur de vie
pub fn update_health_indicator(
    health: Res<LocalPlayerHealth>,
    mut query: Query<&mut Text, With<HealthIndicator>>,
) {
    if let Ok(mut text) = query.get_single_mut() {
        let hearts = match health.health {
            3 => "❤ ❤ ❤",
            2 => "❤ ❤ 🖤",
            1 => "❤ 🖤 🖤",
            _ => "🖤 🖤 🖤",
        };
        text.sections[0].value = hearts.to_string();
    }
}

/// Système pour mettre à jour le scoreboard
pub fn update_scoreboard(
    player_scores: Res<PlayerScores>,
    mut query: Query<&mut Text, With<ScoreboardText>>,
) {
    if let Ok(mut text) = query.get_single_mut() {
        // Trouver le meilleur score
        let top_player = player_scores.scores.iter()
            .max_by_key(|(_, (_, score))| score);

        let top_text = if let Some((_, (name, score))) = top_player {
            format!("Top: {} ({})", name, score)
        } else {
            "Top: -- (0)".to_string()
        };

        // Trouver le score local
        let local_score = if let Some(local_id) = player_scores.local_player_id {
            player_scores.scores.get(&local_id)
                .map(|(_, score)| *score)
                .unwrap_or(0)
        } else {
            0
        };

        text.sections[0].value = format!("{}\nYou: {}", top_text, local_score);
    }
}

/// Système pour générer les tuiles de la minimap (une seule fois)
pub fn generate_minimap_tiles(
    mut commands: Commands,
    mut generated: ResMut<MinimapTilesGenerated>,
    map: Res<crate::network::CurrentMap>,
    minimap_query: Query<Entity, With<Minimap>>,
) {
    // Ne générer qu'une seule fois et seulement si la map est disponible
    if generated.0 {
        return;
    }

    if let Some(game_map) = &map.0 {
        if let Ok(minimap_entity) = minimap_query.get_single() {
            const MINIMAP_SIZE: f32 = 200.0;
            let tile_size = MINIMAP_SIZE / game_map.width as f32;

            // Créer une tuile pour chaque case de la map
            for y in 0..game_map.height {
                for x in 0..game_map.width {
                    let tile_type = game_map.tiles[y][x];

                    // Couleur selon le type de tuile
                    let color = match tile_type as u8 {
                        1 => Color::srgba(0.8, 0.8, 0.8, 0.9), // Mur = gris clair
                        _ => Color::srgba(0.1, 0.1, 0.1, 0.5), // Sol = gris très foncé (transparent)
                    };

                    commands.entity(minimap_entity).with_children(|parent| {
                        parent.spawn((
                            NodeBundle {
                                style: Style {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(x as f32 * tile_size),
                                    top: Val::Px(y as f32 * tile_size),
                                    width: Val::Px(tile_size),
                                    height: Val::Px(tile_size),
                                    ..default()
                                },
                                background_color: color.into(),
                                ..default()
                            },
                            MinimapTile,
                        ));
                    });
                }
            }

            generated.0 = true;
            println!("Minimap tiles generated: {}x{}", game_map.width, game_map.height);
        }
    }
}

/// Système pour mettre à jour la minimap
pub fn update_minimap(
    fps_controller_query: Query<&Transform, With<FpsController>>,
    map: Res<crate::network::CurrentMap>,
    mut minimap_dot_query: Query<&mut Style, With<MinimapPlayerDot>>,
) {
    if let Ok(player_transform) = fps_controller_query.get_single() {
        if let Some(game_map) = &map.0 {
            if let Ok(mut dot_style) = minimap_dot_query.get_single_mut() {
                const MINIMAP_SIZE: f32 = 200.0;

                // Position du joueur dans le monde
                let player_x = player_transform.translation.x;
                let player_z = player_transform.translation.z;

                // Convertir en position sur la minimap (0-200px)
                let minimap_x = (player_x / game_map.width as f32) * MINIMAP_SIZE;
                let minimap_z = (player_z / game_map.height as f32) * MINIMAP_SIZE;

                // Centrer le point (6px de largeur/hauteur)
                dot_style.left = Val::Px(minimap_x - 3.0);
                dot_style.top = Val::Px(minimap_z - 3.0);
            }
        }
    }
}
