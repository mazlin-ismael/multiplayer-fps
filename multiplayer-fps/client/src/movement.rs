use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
use bevy::window::{CursorGrabMode, PrimaryWindow};

use crate::network::CurrentMap;
use shared::TileType;

#[derive(Resource, Default)]
pub struct MapSpawned(pub bool);

#[derive(Resource)]
pub struct CursorState {
    pub locked: bool,
    pub fullscreen: bool,
}

impl Default for CursorState {
    fn default() -> Self {
        Self {
            locked: false,  // Curseur déverrouillé par défaut
            fullscreen: false,
        }
    }
}

pub fn setup_cursor_system(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if let Ok(mut window) = windows.get_single_mut() {
        window.cursor.grab_mode = CursorGrabMode::Locked;
        window.cursor.visible = false;
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
        let floor_cube = meshes.add(Cuboid::new(1.0, 0.05,1.0));

        let wall_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.2, 0.8),
            perceptual_roughness: 0.7,
            ..Default::default()
        });
        let floor_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.5, 0.0), // Orange
            ..Default::default()
        });

        let cx = map.width as f32 / 2.0;
        let cz = map.height as f32 / 2.0;

        // Sol - créer un sol orange en bas avec des cubes aplatis
        for y in 0..map.height {
            for x in 0..map.width {
                commands.spawn(PbrBundle {
                    mesh: floor_cube.clone(),
                    material: floor_mat.clone(),
                    transform: Transform::from_xyz(x as f32, 0.0, y as f32),
                    ..Default::default()
                });
            }
        }

        // Murs
        let mut wall_count = 0;
        for y in 0..map.height {
            for x in 0..map.width {
                if map.tiles[y][x] as u8 == 1 {
                    wall_count += 1;
                    commands.spawn(PbrBundle {
                        mesh: cube.clone(),
                        material: wall_mat.clone(),
                        transform: Transform::from_xyz(x as f32, 2.5, y as f32),
                        ..Default::default()
                    });
                }
            }
        }
        println!("DEBUG: Spawned {} walls", wall_count);

        // Lumière
        commands.spawn(PointLightBundle {
            point_light: PointLight {
                intensity: 2000.0,
                shadows_enabled: true,
                ..Default::default()
            },
            transform: Transform::from_xyz(cx, 8.0, cz),
            ..Default::default()
        });

        spawned.0 = true;
    }
}

/// Caméra
pub fn spawn_camera_system(mut commands: Commands, current_map: Res<CurrentMap>) {
    if current_map.is_changed() && current_map.0.is_some() {
        let map = current_map.0.as_ref().unwrap();
        // Find spawn tile, otherwise fallback to center
        let mut pos = None;
        for y in 0..map.height { for x in 0..map.width { if map.tiles[y][x] == TileType::Spawn { pos = Some((x, y)); break; } } if pos.is_some() { break; } }
        let (x, z) = pos.unwrap_or((map.width/2, map.height/2));
        let cx = x as f32 + 0.5;
        let cz = z as f32 + 0.5;

        commands.spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(cx, 2.0, cz)
                    .looking_at(Vec3::new(cx, 0.0, cz), Vec3::Y),
                ..Default::default()
            },
            FpsController::default(),
        ));
    }
}

#[derive(Component)]
pub struct FpsController {
    pub yaw: f32,
    pub pitch: f32,
    pub speed: f32,
    pub active: bool,
}
impl Default for FpsController {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            speed: 5.0,
            active: true,
        }
    }
}

/// Vérifie si une position collidrait avec un mur
/// Prend en compte la rotation du tank pour faire pivoter les points de collision
fn check_collision_at_position(pos: Vec3, rotation: Quat, map: &shared::GameMap) -> bool {
    // Dimensions du tank (en mètres)
    // CHASSIS: 1.2m large × 1.8m profond
    // CHENILLES: ajoutent 0.125m de chaque côté (0.65 + 0.075 = 0.725m du centre)
    // Total: 1.45m de largeur, 1.8m de profondeur

    // Points de collision en coordonnées LOCALES du tank (avant rotation)
    let local_check_points = [
        Vec3::new(0.0, 0.0, 0.0),      // Centre
        Vec3::new(0.75, 0.0, 0.9),     // Avant-droit (local)
        Vec3::new(-0.75, 0.0, 0.9),    // Avant-gauche (local)
        Vec3::new(0.75, 0.0, -0.9),    // Arrière-droit (local)
        Vec3::new(-0.75, 0.0, -0.9),   // Arrière-gauche (local)
        Vec3::new(0.75, 0.0, 0.0),     // Milieu-droit (local)
        Vec3::new(-0.75, 0.0, 0.0),    // Milieu-gauche (local)
        Vec3::new(0.0, 0.0, 0.9),      // Milieu-avant (local)
        Vec3::new(0.0, 0.0, -0.9),     // Milieu-arrière (local)
    ];

    for local_offset in &local_check_points {
        // IMPORTANT: Transformer le point local en coordonnées monde avec la rotation
        let world_offset = rotation * *local_offset;
        let check_pos = pos + world_offset;
        let tile_x = check_pos.x.floor() as i32;
        let tile_z = check_pos.z.floor() as i32;

        // Vérifier si on est dans les limites de la map
        if tile_x < 0 || tile_x >= map.width as i32 || tile_z < 0 || tile_z >= map.height as i32 {
            return true; // Hors limites = collision
        }

        // Vérifier si c'est un mur
        let tile = map.tiles[tile_z as usize][tile_x as usize];
        if tile as u8 == 1 { // Mur
            return true;
        }
    }

    false // Pas de collision
}

pub fn fps_controller_system(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut query: Query<(&mut Transform, &mut FpsController)>,
    cursor_state: Res<CursorState>,
    current_map: Res<CurrentMap>,
) {
    for (mut transform, mut ctrl) in query.iter_mut() {
        // Sauvegarder la rotation actuelle
        let old_rotation = transform.rotation;

        // rotation seulement si le curseur est verrouillé
        if cursor_state.locked {
            for ev in mouse_motion.read() {
                let sensitivity = 0.002;
                ctrl.yaw -= ev.delta.x * sensitivity;
                ctrl.pitch -= ev.delta.y * sensitivity;
                ctrl.pitch = ctrl.pitch.clamp(-1.54, 1.54);
            }
        }

        let yaw_rot = Quat::from_axis_angle(Vec3::Y, ctrl.yaw);
        let pitch_rot = Quat::from_axis_angle(Vec3::X, ctrl.pitch);
        let new_rotation = yaw_rot * pitch_rot;

        // Vérifier la collision avec la nouvelle rotation
        if let Some(map) = &current_map.0 {
            if check_collision_at_position(transform.translation, new_rotation, map) {
                // La rotation causerait une collision, annuler
                // (on garde old_rotation, donc on ne change pas transform.rotation)
            } else {
                // Rotation OK, l'appliquer
                transform.rotation = new_rotation;
            }
        } else {
            // Pas de map, autoriser la rotation
            transform.rotation = new_rotation;
        }

        // déplacement avec détection de collision
        let mut dir = Vec3::ZERO;
        if keyboard.pressed(KeyCode::KeyW) {
            dir.z -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            dir.z += 1.0;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            dir.x -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            dir.x += 1.0;
        }
        if keyboard.pressed(KeyCode::Space) {
            dir.y += 1.0;
        }
        if keyboard.pressed(KeyCode::ShiftLeft) {
            dir.y -= 1.0;
        }

        if dir.length_squared() > 0.0 {
            let forward = transform.rotation * dir.normalize();
            let movement = forward * ctrl.speed * time.delta_seconds();
            let new_position = transform.translation + movement;

            // Vérifier la collision seulement si on a une map
            if let Some(map) = &current_map.0 {
                if !check_collision_at_position(new_position, transform.rotation, map) {
                    // Pas de collision, on peut bouger
                    transform.translation = new_position;
                }
                // Sinon, on ne bouge pas (collision détectée)
            } else {
                // Pas de map chargée, on autorise le mouvement
                transform.translation = new_position;
            }
        }
    }
}

pub fn cursor_control_system(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut cursor_state: ResMut<CursorState>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if let Ok(mut window) = windows.get_single_mut() {
        // Toggle cursor lock avec Escape
        if keyboard.just_pressed(KeyCode::Escape) {
            cursor_state.locked = !cursor_state.locked;
            
            if cursor_state.locked {
                window.cursor.grab_mode = CursorGrabMode::Locked;
                window.cursor.visible = false;
            } else {
                window.cursor.grab_mode = CursorGrabMode::None;
                window.cursor.visible = true;
            }
        }

        // Toggle fullscreen avec F11
        if keyboard.just_pressed(KeyCode::F11) {
            cursor_state.fullscreen = !cursor_state.fullscreen;
            window.mode = if cursor_state.fullscreen {
                bevy::window::WindowMode::Fullscreen
            } else {
                bevy::window::WindowMode::Windowed
            };
        }
    }
}
