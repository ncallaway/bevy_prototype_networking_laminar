use bevy::prelude::*;

use super::game::{Cube, Message};

use std::net::SocketAddr;

// pub const SERVER: &str = "127.0.0.1:12351";
// pub const CLIENT: &str = "127.0.0.1:12350";

mod prototype;

#[derive(Debug)]
pub struct CubePositionEvent(f32, f32, f32);

#[derive(Debug)]
pub struct ClientUpdateEvent {
    from: String,
    update: String,
}

#[derive(Debug)]
pub struct SyncMessagesEvent {
    messages: Vec<Message>,
}
#[derive(Default)]
pub struct CreateMessages {
    pub messages: Vec<String>,
}

pub enum ConnectionInfo {
    Server {
        addr: SocketAddr,
    },
    Client {
        name: String,
        addr: SocketAddr,
        server: SocketAddr,
    },
}

pub fn build(app: &mut AppBuilder) {
    prototype::build(app);
    app.add_event::<CubePositionEvent>()
        .add_event::<ClientUpdateEvent>()
        .add_event::<SyncMessagesEvent>()
        .init_resource::<EventListenerState>()
        .init_resource::<CreateMessages>()
        .add_resource(parse_args())
        .add_system(handle_cube_events.system())
        .add_system(handle_client_update_events.system())
        .add_system(handle_sync_messages_events.system());
}

#[derive(Default)]
struct EventListenerState {
    cube_events: EventReader<CubePositionEvent>,
    client_update_events: EventReader<ClientUpdateEvent>,
    sync_messages_events: EventReader<SyncMessagesEvent>,
}

// todo: the testbed relies on having just one cube, and the client/server setup being the same
// but a better example might demonstrate stable IDs across the network. To demonstrate
// this consider adding a sphere each client controls, to force us to have to disambiguate.
fn handle_cube_events(
    ci: Res<ConnectionInfo>,
    mut state: ResMut<EventListenerState>,
    cube_events: Res<Events<CubePositionEvent>>,
    mut query: Query<(&Cube, &mut Translation)>,
) {
    if ci.is_server() {
        return;
    }

    for event in state.cube_events.iter(&cube_events) {
        for (_, mut tx) in &mut query.iter() {
            tx.0 = Vec3::new(event.0, event.1, event.2);
        }
    }
}

fn handle_client_update_events(
    mut commands: Commands,
    ci: Res<ConnectionInfo>,
    mut state: ResMut<EventListenerState>,
    client_update_events: Res<Events<ClientUpdateEvent>>,
) {
    if ci.is_client() {
        return;
    }

    for event in state.client_update_events.iter(&client_update_events) {
        commands.spawn((Message::new(
            &event.update,
            &format!("client {}", event.from),
            255,
        ),));
    }
}

fn handle_sync_messages_events(
    mut commands: Commands,
    ci: Res<ConnectionInfo>,
    mut state: ResMut<EventListenerState>,
    sync_messages_events: Res<Events<SyncMessagesEvent>>,
    mut client_messages: Query<(Entity, &mut Message)>,
) {
    if ci.is_server() {
        return;
    }

    for event in state.sync_messages_events.iter(&sync_messages_events) {
        let server_messages = &event.messages;

        let mut client_borrow = client_messages.iter();
        let mut client_iter = client_borrow.into_iter();

        for server_message in &mut server_messages.iter() {
            let has_message = client_iter.next();

            match has_message {
                Some((_, mut client_message)) => {
                    client_message.from = server_message.from.clone();
                    client_message.message = server_message.message.clone();
                    client_message.ordinal = server_message.ordinal;
                }
                None => {
                    commands.spawn((Message::new(
                        &server_message.message,
                        &server_message.from,
                        server_message.ordinal,
                    ),));
                }
            }
        }

        // for (e, _) in client_iter {
        //     commands.despawn(e);
        // }

        // we only process one sync_message_event per frame to avoid double-spawning messages
        break;
    }
}

impl ConnectionInfo {
    pub fn is_server(&self) -> bool {
        return match &self {
            ConnectionInfo::Server { .. } => true,
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

    if args.len() < 3 {
        panic!("Need to provide a port to bind to (e.g. 0.0.0.0:12351)");
    }

    let addr: SocketAddr = args[2]
        .parse()
        .expect("The socket address wasn't a valid format");

    if is_server {
        return ConnectionInfo::Server { addr: addr };
    }

    if args.len() < 4 {
        panic!("When running as a client you need to provide the server port to connect to (e.g. 127.0.0.1:12351)");
    }

    let server_addr: SocketAddr = args[3]
        .parse()
        .expect("The socket address wasn't a valid format");

    if args.len() < 5 {
        panic!("When running as a client a client name needs to be provided.");
    }

    let client_name = &args[4];

    if client_name.len() > 6 {
        panic!("The client name must be < 6 characters");
    }

    return ConnectionInfo::Client {
        name: client_name.clone(),
        addr: addr,
        server: server_addr,
    };
}
