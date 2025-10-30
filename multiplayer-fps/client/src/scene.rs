use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
use bevy_rapier3d::prelude::*;
use bevy_renet::renet::{RenetClient, DefaultChannel};
use shared::{GameMap, ClientMessage};

use crate::network::CurrentMap;
use crate::player_model::create_fps_weapon;

#[derive(Resource, Default)]
pub struct MapSpawned(pub bool);

// Timer pour limiter l'envoi des messages réseau
#[derive(Resource)]
pub struct NetworkUpdateTimer(pub Timer);

impl Default for NetworkUpdateTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.05, TimerMode::Repeating)) // 20 fois par seconde
    }
}

pub fn spawn_map_if_received_system(
    mut commands: Commands,
    mut spawned: ResMut<MapSpawned>,
    current_map: Res<CurrentMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if spawned.0 {
        return;
    }

    if let Some(map) = &current_map.0 {
        info!("Spawning 3D map: {}x{}", map.width, map.height);

        let cube = meshes.add(Cuboid::new(1.0, 5.0, 1.0));
        let plane = meshes.add(Plane3d::default().mesh().size(1.0, 1.0));

        let wall_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.2, 0.8),
            perceptual_roughness: 0.7,
            ..Default::default()
        });
        let floor_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.15, 0.15, 0.15),
            ..Default::default()
        });

        let cx = map.width as f32 / 2.0;
        let cz = map.height as f32 / 2.0;

        // Sol avec collider
        commands.spawn((
            PbrBundle {
                mesh: plane.clone(),
                material: floor_mat.clone(),
                transform: Transform::from_xyz(cx - 0.5, 0.0, cz - 0.5)
                    .with_scale(Vec3::new(map.width as f32, 1.0, map.height as f32)),
                ..Default::default()
            },
            Collider::cuboid(map.width as f32 / 2.0, 0.01, map.height as f32 / 2.0),
        ));

        // Murs avec colliders
        for y in 0..map.height {
            for x in 0..map.width {
                if map.tiles[y][x] as u8 == 1 {
                    commands.spawn((
                        PbrBundle {
                            mesh: cube.clone(),
                            material: wall_mat.clone(),
                            transform: Transform::from_xyz(x as f32, 2.5, y as f32),
                            ..Default::default()
                        },
                        Collider::cuboid(0.5, 2.5, 0.5),
                    ));
                }
            }
        }

        // Lumière ambiante
        commands.spawn(PointLightBundle {
            point_light: PointLight {
                intensity: 3000.0,
                shadows_enabled: true,
                range: 50.0,
                ..Default::default()
            },
            transform: Transform::from_xyz(cx, 10.0, cz),
            ..Default::default()
        });

        // Lumière directionnelle supplémentaire
        commands.spawn(DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 5000.0,
                shadows_enabled: false,
                ..Default::default()
            },
            transform: Transform::from_rotation(Quat::from_euler(
                EulerRot::XYZ,
                -std::f32::consts::FRAC_PI_4,
                std::f32::consts::FRAC_PI_4,
                0.0,
            )),
            ..Default::default()
        });

        spawned.0 = true;
    }
}

/// Spawner la caméra avec contrôleur FPS physique et arme visible
pub fn spawn_camera_system(
    mut commands: Commands,
    current_map: Res<CurrentMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if current_map.is_changed() && current_map.0.is_some() {
        let map = current_map.0.as_ref().unwrap();

        let spawn_x = map.spawn_x;
        let spawn_z = map.spawn_z;

        info!("Spawning player at ({}, {})", spawn_x, spawn_z);

        // Créer l'arme FPS
        let fps_weapon = create_fps_weapon(&mut commands, &mut meshes, &mut materials);

        // Spawner le joueur avec capsule collider + MARKER LocalPlayer
        commands.spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(spawn_x, 1.7, spawn_z),
                ..Default::default()
            },
            FpsController::default(),
            LocalPlayer, // MARKER pour identifier le joueur local
            Collider::capsule_y(0.85, 0.3),
            RigidBody::KinematicPositionBased,
            KinematicCharacterController {
                offset: CharacterLength::Absolute(0.01),
                slide: true,
                autostep: Some(CharacterAutostep {
                    max_height: CharacterLength::Absolute(0.5),
                    min_width: CharacterLength::Absolute(0.2),
                    include_dynamic_bodies: false,
                }),
                ..default()
            },
            Velocity::default(),
        )).add_child(fps_weapon); // Attacher l'arme à la caméra
    }
}

#[derive(Component)]
pub struct LocalPlayer; // Marker pour le joueur local

#[derive(Component)]
pub struct FpsController {
    pub yaw: f32,
    pub pitch: f32,
    pub speed: f32,
}

impl Default for FpsController {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            speed: 5.0,
        }
    }
}

pub fn fps_controller_system(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: EventReader<MouseMotion>,
    windows: Query<&Window>,
    mut query: Query<(
        &mut Transform,
        &mut FpsController,
        &mut KinematicCharacterController,
    ), With<LocalPlayer>>,
) {
    let window_focused = windows
        .get_single()
        .map(|w| w.focused)
        .unwrap_or(false);
    
    let cursor_locked = windows
        .get_single()
        .map(|w| w.cursor.grab_mode == bevy::window::CursorGrabMode::Locked)
        .unwrap_or(false);

    if !window_focused {
        mouse_motion.clear();
        return;
    }

    for (mut transform, mut ctrl, mut controller) in query.iter_mut() {
        // Rotation de la caméra avec la souris SEULEMENT si le curseur est verrouillé
        if cursor_locked {
            for ev in mouse_motion.read() {
                let sensitivity = 0.002;
                ctrl.yaw -= ev.delta.x * sensitivity;
                ctrl.pitch -= ev.delta.y * sensitivity;
                ctrl.pitch = ctrl.pitch.clamp(-1.54, 1.54);
            }

            let yaw_rot = Quat::from_axis_angle(Vec3::Y, ctrl.yaw);
            let pitch_rot = Quat::from_axis_angle(Vec3::X, ctrl.pitch);
            transform.rotation = yaw_rot * pitch_rot;
        } else {
            mouse_motion.clear();
        }

        // Déplacement UNIQUEMENT si le curseur est verrouillé ET la fenêtre a le focus
        if cursor_locked && window_focused {
            let mut movement = Vec3::ZERO;
            
            if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::KeyZ) {
                movement.z -= 1.0;
            }
            if keyboard.pressed(KeyCode::KeyS) {
                movement.z += 1.0;
            }
            if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::KeyQ) {
                movement.x -= 1.0;
            }
            if keyboard.pressed(KeyCode::KeyD) {
                movement.x += 1.0;
            }

            if movement.length_squared() > 0.0 {
                let yaw_only = Quat::from_axis_angle(Vec3::Y, ctrl.yaw);
                let direction = yaw_only * movement.normalize();
                
                let horizontal_movement = Vec3::new(
                    direction.x,
                    0.0,
                    direction.z,
                );
                
                controller.translation = Some(horizontal_movement * ctrl.speed * time.delta_seconds());
            } else {
                controller.translation = None;
            }
        } else {
            controller.translation = None;
        }
    }
}

// NOUVEAU : Envoyer la position au serveur
pub fn send_player_movement_system(
    time: Res<Time>,
    mut timer: ResMut<NetworkUpdateTimer>,
    mut client: ResMut<RenetClient>,
    query: Query<(&Transform, &FpsController), With<LocalPlayer>>,
) {
    timer.0.tick(time.delta());
    
    if timer.0.just_finished() {
        for (transform, controller) in query.iter() {
            let message = ClientMessage::PlayerMovement {
                position: [
                    transform.translation.x,
                    transform.translation.y,
                    transform.translation.z,
                ],
                rotation: [controller.yaw, controller.pitch],
            };
            
            let bytes = message.to_bytes();
            client.send_message(DefaultChannel::ReliableOrdered, bytes);
        }
    }
}