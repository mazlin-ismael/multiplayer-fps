use bevy::prelude::*;
use bevy::input::mouse::MouseButton;
use bevy_renet::renet::{RenetClient, DefaultChannel};
use shared::ClientMessage;
use crate::scene::{FpsController, LocalPlayer};

// Composant pour marquer un projectile
#[derive(Component)]
pub struct Projectile {
    pub velocity: Vec3,
    pub lifetime: Timer,
    pub shooter_id: Option<u64>, // Pour éviter de se toucher soi-même
}

// Resource pour tracker les projectiles du serveur
#[derive(Resource, Default)]
pub struct ServerProjectiles {
    pub projectiles: std::collections::HashMap<u64, Entity>,
}

// Système pour tirer quand on clique
pub fn shoot_system(
    mut commands: Commands,
    mut client: ResMut<RenetClient>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    query: Query<(&Transform, &FpsController), With<LocalPlayer>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
            // Calculer la position de départ du projectile (devant le canon)
            let forward = transform.forward();
            let start_pos = transform.translation + forward * 1.5; // 1.5m devant

            // Direction basée sur yaw + pitch
            let yaw_rot = Quat::from_axis_angle(Vec3::Y, controller.yaw);
            let pitch_rot = Quat::from_axis_angle(Vec3::X, controller.pitch);
            let direction = yaw_rot * pitch_rot * Vec3::NEG_Z;

            // Envoyer le message de tir au serveur
            let message = ClientMessage::Shoot {
                position: [start_pos.x, start_pos.y, start_pos.z],
                direction: [direction.x, direction.y, direction.z],
            };
            client.send_message(DefaultChannel::ReliableOrdered, message.to_bytes());

            // Créer le projectile localement (visuel immédiat)
            spawn_projectile(
                &mut commands,
                &mut meshes,
                &mut materials,
                start_pos,
                direction,
                None, // Le joueur local
            );
        }
    }
}

// Fonction pour créer un projectile visuel
pub fn spawn_projectile(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    direction: Vec3,
    shooter_id: Option<u64>,
) -> Entity {
    let projectile_mesh = meshes.add(Cuboid::new(0.2, 0.2, 0.5)); // Balle 0.2x0.2x0.5 (plus visible)
    let projectile_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.8, 0.0), // Jaune/orange comme un obus
        emissive: Color::srgb(1.0, 0.5, 0.0).into(),
        metallic: 0.8,
        perceptual_roughness: 0.2,
        ..Default::default()
    });

    commands.spawn((
        PbrBundle {
            mesh: projectile_mesh,
            material: projectile_material,
            transform: Transform::from_translation(position)
                .looking_to(direction, Vec3::Y),
            ..Default::default()
        },
        Projectile {
            velocity: direction.normalize() * 50.0, // Vitesse 50 m/s
            lifetime: Timer::from_seconds(5.0, TimerMode::Once), // 5 secondes de vie
            shooter_id,
        },
    )).id()
}

// Système pour déplacer les projectiles
pub fn update_projectiles_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Projectile)>,
) {
    for (entity, mut transform, mut projectile) in query.iter_mut() {
        // Déplacer le projectile
        transform.translation += projectile.velocity * time.delta_seconds();

        // Mettre à jour le lifetime
        projectile.lifetime.tick(time.delta());

        // Supprimer si le lifetime est écoulé
        if projectile.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}

// Système pour recevoir les projectiles des autres joueurs du serveur
pub fn receive_projectiles_system(
    mut commands: Commands,
    mut client: ResMut<RenetClient>,
    mut server_projectiles: ResMut<ServerProjectiles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    while let Some(message) = client.receive_message(DefaultChannel::ReliableOrdered) {
        if let Some(server_msg) = shared::ServerMessage::from_bytes(&message) {
            if let shared::ServerMessage::ProjectileSpawned {
                projectile_id,
                shooter_id,
                position,
                direction,
            } = server_msg {
                // Ne pas créer le projectile si c'est le nôtre (déjà créé localement)
                // TODO: Implémenter la vérification de shooter_id vs notre player_id

                let pos = Vec3::new(position[0], position[1], position[2]);
                let dir = Vec3::new(direction[0], direction[1], direction[2]);

                let entity = spawn_projectile(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    pos,
                    dir,
                    Some(shooter_id),
                );

                server_projectiles.projectiles.insert(projectile_id, entity);
            }
        }
    }
}
