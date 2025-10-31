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

// Système pour tirer quand on clique (raycast instantané)
pub fn shoot_system(
    mut commands: Commands,
    mut client: ResMut<RenetClient>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    query: Query<(&Transform, &FpsController), With<LocalPlayer>>,
    windows: Query<&Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Vérifier que le curseur est verrouillé
    let cursor_locked = windows
        .get_single()
        .map(|w| w.cursor.grab_mode == bevy::window::CursorGrabMode::Locked)
        .unwrap_or(false);

    if !cursor_locked {
        return;
    }

    if mouse_button.just_pressed(MouseButton::Left) {
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
