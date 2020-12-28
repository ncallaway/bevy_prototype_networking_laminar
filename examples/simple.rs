use bevy::app::{ScheduleRunnerPlugin, ScheduleRunnerSettings};
use bevy::prelude::*;

use std::net::SocketAddr;
use std::time::Duration;

use bevy_prototype_networking_laminar::{
    NetworkDelivery, NetworkEvent, NetworkResource, NetworkingPlugin,
};

const SERVER: &str = "127.0.0.1:12351";
const CLIENT: &str = "127.0.0.1:12350";

fn main() {
    App::build()
        .add_plugins(MinimalPlugins)
        .add_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        )))
        .add_plugin(ScheduleRunnerPlugin::default())
        // The NetworkingPlugin
        .add_plugin(NetworkingPlugin)
        // Our send
        .init_resource::<NetworkEventReader>()
        .init_resource::<SendTimer>()
        .add_resource(parse_args())
        .add_startup_system(startup_system.system())
        .add_system(print_network_events.system())
        .add_system(send_messages.system())
        .run();
}

#[derive(Default)]
struct NetworkEventReader {
    network_events: EventReader<NetworkEvent>,
}

fn print_network_events(mut state: ResMut<NetworkEventReader>, events: Res<Events<NetworkEvent>>) {
    for event in state.network_events.iter(&events) {
        match event {
            NetworkEvent::Message(conn, data) => {
                let msg = String::from_utf8_lossy(data);
                println!("<--- {:?} from {}", msg, conn);
            }
            NetworkEvent::Connected(conn) => println!("\tConnected: {}", conn),
            NetworkEvent::Disconnected(conn) => println!("\tDisconnected: {}", conn),
            NetworkEvent::SendError(err) => println!("\tSend Error: {}", err),
        }
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

struct SendTimer {
    message_timer: Timer,
}

impl Default for SendTimer {
    fn default() -> Self {
        SendTimer {
            message_timer: Timer::from_seconds(3.0, true),
        }
    }
}

fn send_messages(
    ci: Res<ConnectionInfo>,
    time: Res<Time>,
    mut state: ResMut<SendTimer>,
    net: ResMut<NetworkResource>,
) {
    state.message_timer.tick(time.delta_seconds());
    if state.message_timer.finished() {
        let server: SocketAddr = SERVER.parse().unwrap();

        let msg = if ci.is_server() {
            "How are things over there?"
        } else {
            "Good."
        };

        println!("---> {:?}", msg);
        if ci.is_server() {
            net.broadcast(msg.as_bytes(), NetworkDelivery::ReliableSequenced(Some(1)))
                .unwrap()
        } else {
            net.send(
                server,
                msg.as_bytes(),
                NetworkDelivery::ReliableSequenced(Some(1)),
            )
            .unwrap()
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
        matches!(&self, ConnectionInfo::Server)
    }

    pub fn is_client(&self) -> bool {
        matches!(&self, ConnectionInfo::Client)
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

    ConnectionInfo::Client
}
