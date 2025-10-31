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

    // Crée une map avec une position de spawn aléatoire
    pub fn with_random_spawn(mut self) -> Self {
        // Générer un spawn aléatoire dans une zone libre
        // On cherche dans toute la map une position où tiles[y][x] == Floor
        use std::time::{SystemTime, UNIX_EPOCH};

        let mut rng_seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        // Simple LCG pour générer des nombres aléatoires
        fn rand(seed: &mut u64) -> u64 {
            *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            *seed
        }

        // Trouver toutes les positions libres (Floor)
        let mut free_positions = Vec::new();
        for y in 1..(self.height - 1) {
            for x in 1..(self.width - 1) {
                if self.tiles[y][x] as u8 == 0 { // Floor
                    // Vérifier que c'est pas trop près d'un mur (au moins 1 case de marge)
                    let safe = self.tiles[y-1][x] as u8 == 0
                            && self.tiles[y+1][x] as u8 == 0
                            && self.tiles[y][x-1] as u8 == 0
                            && self.tiles[y][x+1] as u8 == 0;
                    if safe {
                        free_positions.push((x as f32 + 0.5, y as f32 + 0.5));
                    }
                }
            }
        }

        if !free_positions.is_empty() {
            let index = (rand(&mut rng_seed) as usize) % free_positions.len();
            let (x, z) = free_positions[index];
            self.spawn_x = x;
            self.spawn_z = z;
        } else {
            // Fallback au centre si aucune position libre trouvée
            self.spawn_x = 10.0;
            self.spawn_z = 10.0;
        }

        self
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