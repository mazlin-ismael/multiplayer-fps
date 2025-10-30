pub mod player;
pub mod map;
pub mod network_messages;

pub use player::*;
pub use map::*;
pub use network_messages::*;

pub const PROTOCOL_ID: u64 = 7;