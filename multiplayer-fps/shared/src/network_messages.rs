use serde::{Deserialize, Serialize};

// Messages du CLIENT vers le SERVEUR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    PlayerMovement {
        position: [f32; 3],
        rotation: [f32; 2], // yaw, pitch
    },
}

// Messages du SERVEUR vers les CLIENTS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    // Informer qu'un joueur a rejoint
    PlayerJoined {
        player_id: u64,
        name: String,
        position: [f32; 3],
    },
    // Mise à jour de la position d'un joueur
    PlayerUpdate {
        player_id: u64,
        position: [f32; 3],
        rotation: [f32; 2],
    },
    // Informer qu'un joueur est parti
    PlayerLeft {
        player_id: u64,
    },
    // Envoyer la map (existant déjà)
    MapData {
        data: Vec<u8>,
    },
}

impl ClientMessage {
    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap_or_default()
    }

    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        bincode::deserialize(data).ok()
    }
}

impl ServerMessage {
    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap_or_default()
    }

    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        bincode::deserialize(data).ok()
    }
}