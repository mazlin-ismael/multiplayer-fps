use bevy::prelude::*;
use bevy_renet::renet::{ConnectionConfig, RenetClient, DefaultChannel, transport::{ClientAuthentication, NetcodeClientTransport}};
use std::{net::SocketAddr, net::UdpSocket, time::SystemTime};
use shared::{GameMap, ServerMessage};

#[derive(Resource, Default)]
pub struct ConnectionState {
    pub was_connected: bool,
    pub map_received: bool,
}

#[derive(Resource)]
pub struct CurrentMap(pub Option<GameMap>);

impl Default for CurrentMap {
    fn default() -> Self {
        Self(None)
    }
}

pub fn receive_map_system(
    mut client: ResMut<RenetClient>,
    mut current_map: ResMut<CurrentMap>,
    mut state: ResMut<ConnectionState>,
) {
    if !state.map_received {
        while let Some(message) = client.receive_message(DefaultChannel::ReliableOrdered) {
            if let Some(server_msg) = ServerMessage::from_bytes(&message) {
                if let ServerMessage::MapData { data } = server_msg {
                    println!("DEBUG: Received map data {} bytes", data.len());
                    if let Some(map) = GameMap::from_bytes(&data) {
                        println!("Map received from server!");
                        map.display();
                        current_map.0 = Some(map);
                        state.map_received = true;
                        return; // Sortir après avoir reçu la map
                    } else {
                        println!("ERROR: Failed to parse map from bytes");
                    }
                }
            }
        }
    }
}

fn name_to_user_data(name: &str) -> [u8; 256] {
    let mut user_data = [0u8; 256];
    let bytes = name.as_bytes();
    let len = bytes.len().min(256);
    user_data[..len].copy_from_slice(&bytes[..len]);
    user_data
}

pub fn create_network_resources(addr: String, player_name: String) -> (RenetClient, NetcodeClientTransport) {
    let client = RenetClient::new(ConnectionConfig::default());
    let server_addr: SocketAddr = addr.parse().unwrap();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let client_id = current_time.as_micros() as u64;
    let user_data_bytes = name_to_user_data(&player_name);
    
    let authentication = ClientAuthentication::Unsecure {
        protocol_id: shared::PROTOCOL_ID,
        client_id: client_id,
        server_addr,
        user_data: Some(user_data_bytes),
    };
    
    println!("Connecting to server at {}", server_addr);
    let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();
    (client, transport)
}

pub fn check_connection(
    client: Option<Res<RenetClient>>,
    mut state: ResMut<ConnectionState>,
) {
    if let Some(client) = client {
        let is_connected = client.is_connected();
        if is_connected && !state.was_connected {
            println!("Connected to server!");
        } else if !is_connected && state.was_connected {
            println!("Disconnected from server!");
        }
        state.was_connected = is_connected;
    }
}