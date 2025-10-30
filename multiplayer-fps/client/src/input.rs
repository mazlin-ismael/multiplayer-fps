use std::io::{self, Write};

pub fn get_server_address() -> String {
    print!("Enter server address: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()    
}

pub fn get_player_name() -> String {
    print!("Enter your name: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}
