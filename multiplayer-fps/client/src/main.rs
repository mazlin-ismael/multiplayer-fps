mod network;
mod input;
mod scene;
mod other_players; // NOUVEAU
mod player_model; // Modèles 3D des joueurs
mod shooting; // Système de tir

use bevy::prelude::*;
use bevy_renet::{RenetClientPlugin, transport::NetcodeClientPlugin};
use bevy::window::{CursorGrabMode, PrimaryWindow, WindowMode};
use bevy_rapier3d::prelude::*;

use input::{get_server_address, get_player_name};
use network::{create_network_resources, check_connection, ConnectionState, CurrentMap, receive_map_system};
use scene::{MapSpawned, spawn_map_if_received_system, spawn_camera_system, fps_controller_system, NetworkUpdateTimer, send_player_movement_system};
use other_players::{OtherPlayers, receive_other_players_system, damage_flash_system}; // NOUVEAU
use shooting::{ServerProjectiles, shoot_system, update_projectiles_system}; // Système de tir

fn main() {
    let addr = get_server_address();
    let player_name = get_player_name();
    let (client, transport) = create_network_resources(addr, player_name);

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Multiplayer FPS".to_string(),
                resolution: (1280.0, 720.0).into(),
                mode: WindowMode::Windowed,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RenetClientPlugin)
        .add_plugins(NetcodeClientPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .insert_resource(client)
        .insert_resource(transport)
        .insert_resource(ConnectionState::default())
        .insert_resource(CurrentMap::default())
        .insert_resource(MapSpawned::default())
        .insert_resource(CursorLocked(false))
        .insert_resource(NetworkUpdateTimer::default())
        .insert_resource(OtherPlayers::default()) // NOUVEAU
        .insert_resource(ServerProjectiles::default()) // Système de tir
        .add_systems(Update, handle_cursor_locking)
        .add_systems(Update, toggle_cursor_on_escape)
        .add_systems(Update, lock_on_click)
        .add_systems(Update, toggle_fullscreen)
        .add_systems(Update, receive_map_system)
        .add_systems(Update, receive_other_players_system) // NOUVEAU
        .add_systems(Update, check_connection)
        .add_systems(Update, (spawn_map_if_received_system, spawn_camera_system, fps_controller_system))
        .add_systems(Update, send_player_movement_system)
        .add_systems(Update, shoot_system) // Système de tir
        .add_systems(Update, update_projectiles_system) // Déplacement des projectiles
        .add_systems(Update, damage_flash_system) // Effet visuel de dommage
        .run();
}

#[derive(Resource)]
struct CursorLocked(bool);

fn handle_cursor_locking(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    cursor_locked: Res<CursorLocked>,
) {
    if cursor_locked.is_changed() {
        if let Ok(mut window) = windows.get_single_mut() {
            if cursor_locked.0 {
                window.cursor.grab_mode = CursorGrabMode::Locked;
                window.cursor.visible = false;
            } else {
                window.cursor.grab_mode = CursorGrabMode::None;
                window.cursor.visible = true;
            }
        }
    }
}

fn toggle_cursor_on_escape(
    keys: Res<ButtonInput<KeyCode>>,
    mut cursor_locked: ResMut<CursorLocked>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        cursor_locked.0 = !cursor_locked.0;
        if cursor_locked.0 {
            println!("Cursor locked - Press ESC to unlock");
        } else {
            println!("Cursor unlocked - Click to play");
        }
    }
}

fn lock_on_click(
    mouse: Res<ButtonInput<MouseButton>>,
    mut cursor_locked: ResMut<CursorLocked>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    if let Ok(window) = windows.get_single() {
        if mouse.just_pressed(MouseButton::Left) && window.focused && !cursor_locked.0 {
            cursor_locked.0 = true;
            println!("Cursor locked - Press ESC to unlock");
        }
    }
}

fn toggle_fullscreen(
    keys: Res<ButtonInput<KeyCode>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if keys.just_pressed(KeyCode::F11) {
        if let Ok(mut window) = windows.get_single_mut() {
            window.mode = match window.mode {
                WindowMode::Windowed => {
                    println!("Fullscreen mode (F11)");
                    WindowMode::BorderlessFullscreen
                }
                _ => {
                    println!("Windowed mode (F11)");
                    WindowMode::Windowed
                }
            };
        }
    }
}