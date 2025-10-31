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
            // Crosshair vertical (barre verticale)
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(2.0),
                    height: Val::Px(20.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                background_color: Color::srgba(1.0, 1.0, 1.0, 0.8).into(),
                ..default()
            });

            // Crosshair horizontal (barre horizontale)
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(20.0),
                    height: Val::Px(2.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                background_color: Color::srgba(1.0, 1.0, 1.0, 0.8).into(),
                ..default()
            });

            // Point central
            parent.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Px(4.0),
                        height: Val::Px(4.0),
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    background_color: Color::srgba(1.0, 0.0, 0.0, 0.9).into(),
                    ..default()
                },
                Crosshair,
            ));
        });
}
