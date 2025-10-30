use bevy_renet::renet::{ConnectionConfig, RenetServer, transport::{NetcodeServerTransport, ServerAuthentication, ServerConfig}};
use std::{net::UdpSocket, time::SystemTime};

const ADDR: &str = "127.0.0.1:5000";

pub fn create_network_resources() -> (RenetServer, NetcodeServerTransport) {
    let server = RenetServer::new(ConnectionConfig::default());
    
    let server_addr = ADDR.parse().unwrap();
    let socket = UdpSocket::bind(server_addr).unwrap();
    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    
    let config = ServerConfig {
        current_time,
        max_clients: 10,
        protocol_id: shared::PROTOCOL_ID,
        public_addresses: vec![server_addr],
        authentication: ServerAuthentication::Unsecure,
    };

    println!("Server launch on {}", server_addr);
    let transport = NetcodeServerTransport::new(config, socket).unwrap();
    
    (server, transport)
}