use bevy::prelude::*;
use bevy::input::mouse::MouseButton;
use bevy_renet::renet::{RenetClient, DefaultChannel};
use shared::ClientMessage;
use crate::scene::{FpsController, LocalPlayer};

// Système pour tirer quand on clique (raycast instantané)
pub fn shoot_system(
    mut client: ResMut<RenetClient>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    query: Query<(&Transform, &FpsController), With<LocalPlayer>>,
    windows: Query<&Window>,
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
        }
    }
}

// Les projectiles visuels sont maintenant gérés différemment
// On peut garder un effet visuel de traçage si nécessaire

