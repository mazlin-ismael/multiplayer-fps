use bevy::prelude::*;
use bevy::input::mouse::MouseButton;
use bevy_renet::renet::{RenetClient, DefaultChannel};
use shared::ClientMessage;
use crate::scene::{FpsController, LocalPlayer};

// Marker pour les projectiles visuels
#[derive(Component)]
pub struct VisualProjectile {
    pub direction: Vec3,
    pub speed: f32,
    pub lifetime: Timer,
}

// Resource pour gérer le cooldown de tir
#[derive(Resource)]
pub struct ShootCooldown {
    pub timer: Timer,
}

impl Default for ShootCooldown {
    fn default() -> Self {
        let mut timer = Timer::from_seconds(0.5, TimerMode::Once);
        timer.set_elapsed(timer.duration()); // Commence prêt
        Self { timer }
    }
}

// Marker pour l'indicateur de reload UI
#[derive(Component)]
pub struct ReloadIndicator;

// Système pour créer l'indicateur de reload
pub fn setup_reload_indicator(mut commands: Commands) {
    // Barre de reload en bas au centre de l'écran
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexEnd,
            padding: UiRect::bottom(Val::Px(80.0)), // 80px du bas
            ..default()
        },
        ..default()
    })
    .with_children(|parent| {
        // Conteneur de la barre
        parent.spawn(NodeBundle {
            style: Style {
                width: Val::Px(150.0),
                height: Val::Px(8.0),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            background_color: Color::srgba(0.1, 0.1, 0.1, 0.8).into(),
            border_color: Color::srgb(0.3, 0.3, 0.3).into(),
            ..default()
        })
        .with_children(|bar_parent| {
            // Barre de progression
            bar_parent.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    background_color: Color::srgb(0.0, 0.8, 0.2).into(), // Vert
                    ..default()
                },
                ReloadIndicator,
            ));
        });
    });
}

// Système pour mettre à jour l'indicateur de reload
pub fn update_reload_indicator(
    cooldown: Res<ShootCooldown>,
    mut query: Query<(&mut Style, &mut BackgroundColor), With<ReloadIndicator>>,
) {
    if let Ok((mut style, mut color)) = query.get_single_mut() {
        let progress = cooldown.timer.fraction();

        // Largeur de la barre = progression
        style.width = Val::Percent(progress * 100.0);

        // Couleur: rouge si pas prêt, vert si prêt
        if cooldown.timer.finished() {
            *color = Color::srgb(0.0, 0.8, 0.2).into(); // Vert
        } else {
            *color = Color::srgb(0.8, 0.2, 0.0).into(); // Rouge
        }
    }
}

// Système pour tirer quand on clique (raycast instantané)
pub fn shoot_system(
    mut commands: Commands,
    mut client: ResMut<RenetClient>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    query: Query<(&Transform, &FpsController), With<LocalPlayer>>,
    windows: Query<&Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
    mut cooldown: ResMut<ShootCooldown>,
) {
    // Mettre à jour le timer
    cooldown.timer.tick(time.delta());

    // Vérifier que le curseur est verrouillé
    let cursor_locked = windows
        .get_single()
        .map(|w| w.cursor.grab_mode == bevy::window::CursorGrabMode::Locked)
        .unwrap_or(false);

    if !cursor_locked {
        return;
    }

    // Vérifier le cooldown ET le clic
    if mouse_button.just_pressed(MouseButton::Left) && cooldown.timer.finished() {
        // Réinitialiser le timer
        cooldown.timer.reset();

        for (transform, controller) in query.iter() {
            // IMPORTANT: Le tir part du CENTRE DE L'ÉCRAN (où est le crosshair)
            // Le canon visible en bas de l'écran est purement esthétique

            // Calculer la direction du tir depuis le crosshair (direction du regard)
            let yaw_rot = Quat::from_axis_angle(Vec3::Y, controller.yaw);
            let pitch_rot = Quat::from_axis_angle(Vec3::X, controller.pitch);
            let direction = yaw_rot * pitch_rot * Vec3::NEG_Z;

            // Position de départ: position de la caméra (au centre de l'écran)
            // C'est exactement où le crosshair pointe
            let start_pos = transform.translation;

            println!("Shooting raycast from CAMERA CENTER {:?} in direction {:?}", start_pos, direction);

            // Envoyer le message de tir au serveur (raycast)
            let message = ClientMessage::Shoot {
                position: [start_pos.x, start_pos.y, start_pos.z],
                direction: [direction.x, direction.y, direction.z],
            };
            client.send_message(DefaultChannel::ReliableOrdered, message.to_bytes());

            // Créer un projectile visuel pour montrer la trajectoire
            let projectile_mesh = meshes.add(Sphere::new(0.05)); // Petite sphère de 5cm
            let projectile_material = materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.8, 0.0), // Jaune/orange
                emissive: Color::srgb(2.0, 1.5, 0.0).into(), // Émet de la lumière
                ..Default::default()
            });

            commands.spawn((
                PbrBundle {
                    mesh: projectile_mesh,
                    material: projectile_material,
                    transform: Transform::from_translation(start_pos),
                    ..Default::default()
                },
                VisualProjectile {
                    direction: direction.normalize(),
                    speed: 100.0, // 100 m/s
                    lifetime: Timer::from_seconds(2.0, TimerMode::Once), // Disparaît après 2s
                },
            ));
        }
    }
}

// Système pour animer les projectiles visuels
pub fn update_visual_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut projectiles: Query<(Entity, &mut Transform, &mut VisualProjectile)>,
) {
    for (entity, mut transform, mut projectile) in projectiles.iter_mut() {
        // Mettre à jour le timer
        projectile.lifetime.tick(time.delta());

        // Si le projectile a expiré, le supprimer
        if projectile.lifetime.finished() {
            commands.entity(entity).despawn();
            continue;
        }

        // Déplacer le projectile
        transform.translation += projectile.direction * projectile.speed * time.delta_seconds();
    }
}
