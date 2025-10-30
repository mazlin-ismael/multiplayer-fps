use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerId(pub u64);

#[derive(Component, Debug, Clone)]
pub struct PlayerName(pub String);


#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerPosition {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Default for PlayerPosition {
    fn default() -> Self {
        Self { x : 0.0, y: 0.0, z: 0.0 }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerVelocity {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Default for PlayerVelocity {
    fn default() -> Self {
        Self { x : 0.0, y: 0.0, z: 0.0 }
    }
}


#[derive(Bundle)]
pub struct PlayerBundle {
    pub id: PlayerId,
    pub name: PlayerName,
    pub position: PlayerPosition,
    pub velocity: PlayerVelocity,
}

impl PlayerBundle {
    pub fn new(id: u64, name: String) -> Self {
        Self {
            id: PlayerId(id),
            name: PlayerName(name),
            position: PlayerPosition::default(),
            velocity: PlayerVelocity::default(),
        }
    }
}