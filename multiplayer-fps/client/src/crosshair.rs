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
    const LINE_LENGTH: f32 = 18.0;  // Longueur de chaque segment (plus long)
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
                position_type: PositionType::Absolute,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            // Segment HAUT
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(LINE_THICKNESS),
                    height: Val::Px(LINE_LENGTH),
                    position_type: PositionType::Absolute,
                    top: Val::Px(-GAP - LINE_LENGTH), // Au-dessus du centre
                    ..default()
                },
                background_color: Color::srgba(1.0, 1.0, 1.0, 0.8).into(),
                ..default()
            });

            // Segment BAS
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(LINE_THICKNESS),
                    height: Val::Px(LINE_LENGTH),
                    position_type: PositionType::Absolute,
                    top: Val::Px(GAP), // En-dessous du centre
                    ..default()
                },
                background_color: Color::srgba(1.0, 1.0, 1.0, 0.8).into(),
                ..default()
            });

            // Segment GAUCHE
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(LINE_LENGTH),
                    height: Val::Px(LINE_THICKNESS),
                    position_type: PositionType::Absolute,
                    left: Val::Px(-GAP - LINE_LENGTH), // À gauche du centre
                    ..default()
                },
                background_color: Color::srgba(1.0, 1.0, 1.0, 0.8).into(),
                ..default()
            });

            // Segment DROITE
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(LINE_LENGTH),
                    height: Val::Px(LINE_THICKNESS),
                    position_type: PositionType::Absolute,
                    left: Val::Px(GAP), // À droite du centre
                    ..default()
                },
                background_color: Color::srgba(1.0, 1.0, 1.0, 0.8).into(),
                ..default()
            });
        });
}
