use bevy::prelude::*;

// Marker pour le crosshair
#[derive(Component)]
pub struct Crosshair;

// Resource pour suivre si le crosshair a été créé
#[derive(Resource, Default)]
pub struct CrosshairSpawned(pub bool);

// Système pour créer le crosshair au centre de l'écran (après la caméra)
pub fn setup_crosshair(
    mut commands: Commands,
    mut spawned: ResMut<CrosshairSpawned>,
    camera_query: Query<&Camera>,
) {
    // Ne créer le crosshair que si la caméra existe et qu'il n'a pas déjà été créé
    if spawned.0 || camera_query.is_empty() {
        return;
    }

    spawned.0 = true;

    // Paramètres du crosshair - lignes longues avec petit espace
    const LINE_LENGTH: f32 = 18.0;  // Longueur de chaque segment
    const LINE_THICKNESS: f32 = 2.0; // Épaisseur
    const GAP: f32 = 2.0;           // Petit espace au centre

    // Créer le crosshair au centre de l'écran
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            // Point central de référence (0x0) - tous les segments sont positionnés par rapport à ce point
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(0.0),
                    height: Val::Px(0.0),
                    position_type: PositionType::Relative,
                    ..default()
                },
                ..default()
            })
            .with_children(|center| {
                // Segment HAUT (vertical)
                center.spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(LINE_THICKNESS),
                        height: Val::Px(LINE_LENGTH),
                        position_type: PositionType::Absolute,
                        left: Val::Px(-LINE_THICKNESS / 2.0), // Centré horizontalement
                        top: Val::Px(-(GAP + LINE_LENGTH)),   // Au-dessus avec gap
                        ..default()
                    },
                    background_color: Color::srgba(1.0, 1.0, 1.0, 0.8).into(),
                    ..default()
                });

                // Segment BAS (vertical)
                center.spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(LINE_THICKNESS),
                        height: Val::Px(LINE_LENGTH),
                        position_type: PositionType::Absolute,
                        left: Val::Px(-LINE_THICKNESS / 2.0), // Centré horizontalement
                        top: Val::Px(GAP),                     // En-dessous avec gap
                        ..default()
                    },
                    background_color: Color::srgba(1.0, 1.0, 1.0, 0.8).into(),
                    ..default()
                });

                // Segment GAUCHE (horizontal)
                center.spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(LINE_LENGTH),
                        height: Val::Px(LINE_THICKNESS),
                        position_type: PositionType::Absolute,
                        left: Val::Px(-(GAP + LINE_LENGTH)),   // À gauche avec gap
                        top: Val::Px(-LINE_THICKNESS / 2.0),  // Centré verticalement
                        ..default()
                    },
                    background_color: Color::srgba(1.0, 1.0, 1.0, 0.8).into(),
                    ..default()
                });

                // Segment DROITE (horizontal)
                center.spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(LINE_LENGTH),
                        height: Val::Px(LINE_THICKNESS),
                        position_type: PositionType::Absolute,
                        left: Val::Px(GAP),                    // À droite avec gap
                        top: Val::Px(-LINE_THICKNESS / 2.0),  // Centré verticalement
                        ..default()
                    },
                    background_color: Color::srgba(1.0, 1.0, 1.0, 0.8).into(),
                    ..default()
                });
            });
        });
}
