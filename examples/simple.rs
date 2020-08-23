use bevy::prelude::*;

use bevy_prototype_laminar_networking::{
    NetworkDelivery, NetworkEvent, NetworkResource, NetworkingPlugin,
};

use std::net::SocketAddr;

const SERVER: &str = "127.0.0.1:12351";
const CLIENT: &str = "127.0.0.1:12350";

fn main() {
    // // EXAMPLE: using the prototype plugin
    App::build()
        .init_resource::<EventListenerState>()
        .init_resource::<MessageTimerState>()
        .add_resource(parse_args())
        .add_default_plugins()
        .add_plugin(NetworkingPlugin)
        .add_startup_system(startup_system.system())
        .add_system(print_network_events.system())
        .add_system(send_messages.system())
        .run();
}

#[derive(Default)]
struct EventListenerState {
    network_events: EventReader<NetworkEvent>,
}

fn print_network_events(
    mut state: ResMut<EventListenerState>,
    my_events: Res<Events<NetworkEvent>>,
) {
    for event in state.network_events.iter(&my_events) {
        println!("Network Event: {:?}", event);
    }
}

fn startup_system(net: ResMut<NetworkResource>, ci: Res<ConnectionInfo>) {
    if ci.is_server() {
        start_server(net);
    } else {
        start_client(net);
    }
}

fn start_server(mut net: ResMut<NetworkResource>) {
    net.bind(SERVER).unwrap();
}

fn start_client(mut net: ResMut<NetworkResource>) {
    net.bind(CLIENT).unwrap();
}

struct MessageTimerState {
    message_timer: Timer,
}

impl Default for MessageTimerState {
    fn default() -> Self {
        MessageTimerState {
            message_timer: Timer::from_seconds(5.0),
        }
    }
}

fn send_messages(
    ci: Res<ConnectionInfo>,
    time: Res<Time>,
    mut state: ResMut<MessageTimerState>,
    net: ResMut<NetworkResource>,
) {
    state.message_timer.tick(time.delta_seconds);
    if state.message_timer.finished {
        let server: SocketAddr = SERVER.parse().unwrap();

        if ci.is_server() {
            net.broadcast(
                b"How are things over there?",
                NetworkDelivery::ReliableSequenced(Some(1)),
            );
        } else {
            net.send(
                server,
                b"Good.",
                NetworkDelivery::ReliableSequenced(Some(1)),
            );
        }
        state.message_timer.reset();
    }
}

pub enum ConnectionInfo {
    Server,
    Client,
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
            ConnectionInfo::Client => true,
            _ => false,
        };
    }
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

    return ConnectionInfo::Client;
}
