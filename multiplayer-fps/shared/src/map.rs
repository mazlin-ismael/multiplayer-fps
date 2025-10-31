use serde::{Deserialize, Serialize};
use bevy::prelude::Resource;

// Represents a tile type: Floor or Wall
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TileType {
    Floor = 0,
    Wall = 1,
}

impl TileType {
    // Converts a digit (0 or 1) to TileType
    pub fn from_digit(d: u8) -> Self {
        if d == 1 { TileType::Wall } else { TileType::Floor }
    }
}

// Structure that contains the entire map
#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct GameMap {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<TileType>>,
    pub spawn_x: f32,
    pub spawn_z: f32,
}

// Global map - Edit here to change the map
pub const MAP_DATA: &[&[u8]] = &[
    &[1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    &[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
    &[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
    &[1, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1],
    &[1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1],
    &[1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1],
    &[1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1],
    &[1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1],
    &[1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1],
    &[1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1],
    &[1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1],
    &[1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1],
    &[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
    &[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
    &[1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1],
    &[1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1],
    &[1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1],
    &[1, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1],
    &[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
    &[1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
];

impl GameMap {
    // Loads map from MAP_DATA (server side) with default spawn
    pub fn from_global() -> Self {
        let height = MAP_DATA.len();
        let width = MAP_DATA[0].len();
        let mut tiles = vec![vec![TileType::Floor; width]; height];
        
        for (y, row) in MAP_DATA.iter().enumerate() {
            for (x, &tile) in row.iter().enumerate() {
                tiles[y][x] = TileType::from_digit(tile);
            }
        }
        
        Self {
            width,
            height,
            tiles,
            spawn_x: 10.0,
            spawn_z: 10.0,
        }
    }

    // Crée une map avec une position de spawn spécifique
    pub fn with_spawn_position(mut self, spawn_index: usize) -> Self {
        // Positions de spawn prédéfinies (espaces libres)
        let spawn_positions = vec![
            (2.5, 2.5),   // Coin haut-gauche
            (17.5, 2.5),  // Coin haut-droite
            (2.5, 17.5),  // Coin bas-gauche
            (17.5, 17.5), // Coin bas-droite
            (10.0, 2.5),  // Centre-haut
            (10.0, 17.5), // Centre-bas
            (2.5, 10.0),  // Centre-gauche
            (17.5, 10.0), // Centre-droite
            (10.0, 10.0), // Centre-centre
        ];

        let (x, z) = spawn_positions[spawn_index % spawn_positions.len()];
        self.spawn_x = x;
        self.spawn_z = z;
        self
    }

    // Converts to bytes for network transmission
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }

    // Reconstructs from received bytes (client side)
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        serde_json::from_slice(bytes).ok()
    }

    pub fn display(&self) {
        println!("\n=== MAP {}x{} ===", self.width, self.height);
        println!("Spawn position: ({:.1}, {:.1})", self.spawn_x, self.spawn_z);
        for row in &self.tiles {
            for tile in row {
                print!("{}", if *tile as u8 == 1 { '#' } else { '.' });
            }
            println!();
        }
        println!("==================\n");
    }
}