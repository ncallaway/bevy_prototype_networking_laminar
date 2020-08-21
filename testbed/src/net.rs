use bevy::prelude::*;

pub enum ConnectionInfo {
    Server,
    Client { name: String },
}

impl ConnectionInfo {
    pub fn is_server(&self) -> bool {
        return match &self {
            ConnectionInfo::Server => true,
            _ => false,
        };
    }

    pub fn is_client(&self) -> bool {
        return match &self {
            ConnectionInfo::Client { .. } => true,
            _ => false,
        };
    }
}

pub fn build(app: &mut AppBuilder) {
    app.add_resource(parse_args());
}

fn parse_args() -> ConnectionInfo {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        panic!("Need to select to run as either a server (--server) or a client (--client).");
    }

    let connection_type = &args[1];

    let is_server = match connection_type.as_str() {
        "--server" | "-s" => true,
        "--client" | "-c" => false,
        _ => panic!("Need to select to run as either a server (--server) or a client (--client)."),
    };

    if is_server {
        return ConnectionInfo::Server;
    }

    if args.len() < 3 {
        panic!("When running as a client a client name needs to be provided.");
    }

    if args[2].len() > 6 {
        panic!("The client name must be < 6 characters");
    }

    return ConnectionInfo::Client {
        name: args[2].clone(),
    };
}
