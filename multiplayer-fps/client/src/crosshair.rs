use bevy::prelude::*;

// Marker pour le crosshair
#[derive(Component)]
pub struct Crosshair;

// Système pour créer le crosshair au centre de l'écran
pub fn setup_crosshair(mut commands: Commands) {
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
