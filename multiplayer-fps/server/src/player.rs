use bevy::prelude::*;
use shared::PlayerBundle;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Projectile {
    pub id: u64,
    pub shooter_id: u64,
    pub position: Vec3,
    pub velocity: Vec3,
    pub lifetime: f32,
}

#[derive(Resource, Default)]
pub struct ProjectileRegistry {
    pub projectiles: HashMap<u64, Projectile>,
    pub next_id: u64,
}

impl ProjectileRegistry {
    pub fn spawn_projectile(&mut self, shooter_id: u64, position: Vec3, direction: Vec3) -> u64 {
        let projectile_id = self.next_id;
        self.next_id += 1;

        let projectile = Projectile {
            id: projectile_id,
            shooter_id,
            position,
            velocity: direction.normalize() * 50.0, // 50 m/s
            lifetime: 5.0, // 5 secondes
        };

        self.projectiles.insert(projectile_id, projectile);
        projectile_id
    }
}

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub entity: Entity,
    pub name: String,
    pub position: [f32; 3],
    pub rotation: [f32; 2], // yaw, pitch
    pub health: u8, // Points de vie (max 3)
}

#[derive(Resource, Default)]
pub struct PlayerRegistry {
    pub players: HashMap<u64, PlayerState>, // player_id -> state
    pub temp_to_player: HashMap<u64, u64>,  // temp_id -> player_id
    pub next_id: u64,
    pub spawn_assignments: HashMap<u64, usize>,
}

impl PlayerRegistry {
    pub fn add_player(&mut self, temp_id: u64, name: String, commands: &mut Commands) -> u64 {
        let player_id = self.next_id;
        self.next_id += 1;

        let entity = commands.spawn(PlayerBundle::new(player_id, name.clone())).id();
        
        // Assigner un index de spawn
        let spawn_index = self.spawn_assignments.len();
        self.spawn_assignments.insert(player_id, spawn_index);

        // Créer le state du joueur
        let state = PlayerState {
            entity,
            name: name.clone(),
            position: [0.0, 0.0, 0.0], // Sera mis à jour
            rotation: [0.0, 0.0],
            health: 3, // 3 points de vie au départ
        };

        self.players.insert(player_id, state);
        self.temp_to_player.insert(temp_id, player_id);

        println!("=== NEW PLAYER ===");
        println!("Name: {}", name);
        println!("Temp ID (network): {}", temp_id);
        println!("Player ID (game): {}", player_id);
        println!("Spawn index: {}", spawn_index);
        println!("Total players: {}", self.players.len());
        println!("==================");

        player_id
    }

    pub fn get_spawn_index(&self, player_id: u64) -> Option<usize> {
        self.spawn_assignments.get(&player_id).copied()
    }

    pub fn get_player_id_from_temp(&self, temp_id: u64) -> Option<u64> {
        self.temp_to_player.get(&temp_id).copied()
    }

    pub fn update_player_position(&mut self, player_id: u64, position: [f32; 3], rotation: [f32; 2]) {
        if let Some(state) = self.players.get_mut(&player_id) {
            state.position = position;
            state.rotation = rotation;
        }
    }

    pub fn get_all_players_except(&self, exclude_id: u64) -> Vec<(u64, &PlayerState)> {
        self.players
            .iter()
            .filter(|(id, _)| **id != exclude_id)
            .map(|(id, state)| (*id, state))
            .collect()
    }

    pub fn remove_player(&mut self, temp_id: u64, commands: &mut Commands) {
        if let Some(player_id) = self.temp_to_player.remove(&temp_id) {
            if let Some(state) = self.players.remove(&player_id) {
                self.spawn_assignments.remove(&player_id);
                commands.entity(state.entity).despawn();
                println!("Player {} removed (temp_id: {})", player_id, temp_id);
            }
        }
    }

    // Infliger des dégâts à un joueur
    pub fn damage_player(&mut self, player_id: u64, damage: u8) -> Option<(u8, bool)> {
        if let Some(state) = self.players.get_mut(&player_id) {
            state.health = state.health.saturating_sub(damage);
            let is_dead = state.health == 0;
            return Some((state.health, is_dead));
        }
        None
    }

    // Soigner un joueur
    pub fn heal_player(&mut self, player_id: u64, amount: u8) {
        if let Some(state) = self.players.get_mut(&player_id) {
            state.health = (state.health + amount).min(3); // Max 3 coeurs
        }
    }

    // Respawn un joueur
    pub fn respawn_player(&mut self, player_id: u64, position: [f32; 3]) {
        if let Some(state) = self.players.get_mut(&player_id) {
            state.health = 3;
            state.position = position;
        }
    }
}

pub fn extract_name_from_user_data(user_data: &[u8; 256]) -> String {
    let end = user_data.iter().position(|&b| b == 0).unwrap_or(256);
    String::from_utf8_lossy(&user_data[..end]).to_string()
}