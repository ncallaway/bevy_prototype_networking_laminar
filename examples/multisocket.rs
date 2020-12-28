use bevy::prelude::*;

use bevy_prototype_networking_laminar::{
    NetworkDelivery, NetworkEvent, NetworkResource, NetworkingPlugin, SendConfig, SocketHandle,
};

use bevy::app::{ScheduleRunnerPlugin, ScheduleRunnerSettings};
use std::net::SocketAddr;
use std::time::Duration;

const SERVER: &str = "127.0.0.1:12351";
const CLIENT: &str = "127.0.0.1:12350";

fn main() {
    App::build()
        .add_plugins(MinimalPlugins)
        .add_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        )))
        .add_plugin(ScheduleRunnerPlugin::default())
        .add_plugin(NetworkingPlugin)
        .init_resource::<EventListenerState>()
        .init_resource::<SendTimerState>()
        .init_resource::<Sockets>()
        .add_startup_system(startup.system())
        .add_system(print_messages.system())
        .add_system(send_messages.system())
        .run();
}

fn startup(mut net: ResMut<NetworkResource>, mut sockets: ResMut<Sockets>) {
    sockets.server = Some(net.bind(SERVER).unwrap());
    sockets.client = Some(net.bind(CLIENT).unwrap());
}

fn send_messages(
    time: Res<Time>,
    sockets: Res<Sockets>,
    mut state: ResMut<SendTimerState>,
    net: ResMut<NetworkResource>,
) {
    state.message_timer.tick(time.delta_seconds());
    if state.message_timer.finished() {
        let server: SocketAddr = SERVER.parse().unwrap();
        let client: SocketAddr = CLIENT.parse().unwrap();

        let (to, who, message, socket, from_server) = if state.from_server {
            (client, "SERVER", "How are things?", sockets.server, false)
        } else {
            (server, "CLIENT", "Good. Thanks!", sockets.client, true)
        };

        println!("[{}] ---> {:?}", who, message);
        let _ = net.send_with_config(
            to,
            message.as_bytes(),
            NetworkDelivery::ReliableSequenced(Some(1)),
            SendConfig { socket },
        );
        state.from_server = from_server;

        state.message_timer.reset();
    }
}

fn print_messages(
    mut state: ResMut<EventListenerState>,
    sockets: Res<Sockets>,
    my_events: Res<Events<NetworkEvent>>,
) {
    if let Some(server) = sockets.server {
        if let Some(client) = sockets.client {
            for event in state.network_events.iter(&my_events) {
                match event {
                    NetworkEvent::Message(conn, data) => {
                        let msg = String::from_utf8_lossy(data);

                        let from = if conn.socket == server {
                            "SERVER"
                        } else if conn.socket == client {
                            "CLIENT"
                        } else {
                            "UNKNOWN"
                        };

                        println!("\t ---> [{}] {:?}\n", from, msg);
                    }
                    NetworkEvent::Connected(conn) => println!("\t {} connected", conn),
                    _ => {}
                }
            }
        }
    }
}

#[derive(Default)]
struct Sockets {
    server: Option<SocketHandle>,
    client: Option<SocketHandle>,
}

#[derive(Default)]
struct EventListenerState {
    network_events: EventReader<NetworkEvent>,
}

struct SendTimerState {
    message_timer: Timer,
    from_server: bool,
}

impl Default for SendTimerState {
    fn default() -> Self {
        SendTimerState {
            message_timer: Timer::new(Duration::from_secs_f64(2.5), true),
            from_server: true,
        }
    }
}
