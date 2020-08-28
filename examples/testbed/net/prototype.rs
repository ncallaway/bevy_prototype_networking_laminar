use bevy::prelude::*;

use bevy_prototype_networking_laminar::{
    Connection, NetworkDelivery, NetworkEvent, NetworkResource,
};

use serde::{Deserialize, Serialize};

use std::net::SocketAddr;

use super::super::game::{Cube, Message};
use super::{
    ClientUpdateEvent, ConnectionInfo, CreateMessages, CubePositionEvent, SyncMessagesEvent,
};

// if false, we will serialize using bincode instead
const SERIALIZE_JSON: bool = true;

pub fn build(app: &mut AppBuilder) {
    app.init_resource::<NetworkEventState>()
        .init_resource::<Players>()
        .add_stage_after(stage::UPDATE, "prototype_after")
        .add_startup_system(initial_connection_system.system())
        .add_system(send_cube_position_system.system())
        .add_system(handle_network_events.system())
        .add_system(send_create_message_system.system())
        .add_system_to_stage("prototype_after", send_message_update_system.system());
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
enum TestbedMessage {
    Introduction(String),
    Pong,
    Disconnecting,
    CubePosition(f32, f32, f32),
    CreateMessage(String),
    SyncMessages { messages: Vec<Message> },
}

fn initial_connection_system(ci: Res<ConnectionInfo>, net: ResMut<NetworkResource>) {
    match &(*ci) {
        ConnectionInfo::Server { addr } => start_server(*addr, net),
        ConnectionInfo::Client { name, addr, server } => start_client(&name, *addr, *server, net),
    }
}

fn send_message_update_system(
    ci: Res<ConnectionInfo>,
    net: Res<NetworkResource>,
    mut changed_query: Query<(Changed<Message>,)>,
    all_messages_query: Query<&Message>,
) {
    if ci.is_client() {
        return;
    }

    if changed_query.iter().into_iter().next().is_some() {
        broadcast_all_messages(net, all_messages_query);
    }
}

// todo: separate network tick rate from game tick rate
fn send_cube_position_system(
    ci: Res<ConnectionInfo>,
    net: Res<NetworkResource>,
    mut query: Query<(&Cube, &Translation)>,
) {
    if ci.is_client() {
        return;
    }

    for (_, tx) in &mut query.iter() {
        let pos = tx.0;

        let msg = TestbedMessage::CubePosition(pos.x(), pos.y(), pos.z());
        let _ = net
            .broadcast(
                &msg.encode()[..],
                NetworkDelivery::UnreliableSequenced(Some(1)),
            )
            .unwrap();
    }
}

fn send_create_message_system(
    ci: Res<ConnectionInfo>,
    net: Res<NetworkResource>,
    mut network_create_messages: ResMut<CreateMessages>,
) {
    if let ConnectionInfo::Client { server, .. } = &(*ci) {
        for msg in &network_create_messages.messages {
            net.send(
                *server,
                &TestbedMessage::CreateMessage(msg.clone()).encode()[..],
                NetworkDelivery::ReliableOrdered(Some(1)),
            )
            .expect("Create message failed to send");
        }
    }

    network_create_messages.messages.clear();
}

#[derive(Default)]
struct NetworkEventState {
    network_events: EventReader<NetworkEvent>,
}

#[derive(Default)]
struct Players {
    names: std::collections::HashMap<Connection, String>,
}

#[allow(clippy::too_many_arguments)]
fn handle_network_events(
    ci: Res<ConnectionInfo>,
    net: Res<NetworkResource>,
    mut state: ResMut<NetworkEventState>,
    network_events: Res<Events<NetworkEvent>>,
    mut cube_events: ResMut<Events<CubePositionEvent>>,
    mut client_update_events: ResMut<Events<ClientUpdateEvent>>,
    mut sync_messages_events: ResMut<Events<SyncMessagesEvent>>,
    mut players: ResMut<Players>,
) {
    for event in state.network_events.iter(&network_events) {
        match event {
            NetworkEvent::Message(conn, msg) => {
                let msg = TestbedMessage::decode(&msg[..]);
                match msg {
                    TestbedMessage::Introduction(name) => handle_introduction_event(
                        name,
                        *conn,
                        &net,
                        &mut players,
                        &mut client_update_events,
                    ),
                    TestbedMessage::CubePosition(x, y, z) => {
                        handle_cube_position_event(x, y, z, &mut cube_events)
                    }
                    TestbedMessage::CreateMessage(msg) => handle_create_message_event(
                        msg,
                        *conn,
                        &ci,
                        &mut client_update_events,
                        &mut players,
                    ),
                    TestbedMessage::SyncMessages { messages } => {
                        handle_sync_messages_event(messages, &ci, &mut sync_messages_events)
                    }
                    _ => {}
                }
            }
            NetworkEvent::Disconnected(conn) => {
                if let Some(name) = players.names.remove(conn) {
                    client_update_events.send(ClientUpdateEvent {
                        from: name,
                        update: "disconnected".to_string(),
                    });
                }
            }
            _ => {}
        }
    }
}

fn handle_introduction_event(
    name: String,
    conn: Connection,
    net: &Res<NetworkResource>,
    players: &mut ResMut<Players>,
    client_update_events: &mut ResMut<Events<ClientUpdateEvent>>,
) {
    let _ = net.send(
        conn.addr,
        &TestbedMessage::Pong.encode()[..],
        NetworkDelivery::ReliableSequenced(Some(2)),
    );
    players.names.insert(conn, name.clone());
    client_update_events.send(ClientUpdateEvent {
        from: name,
        update: "connected".to_string(),
    });
}

fn handle_cube_position_event(
    x: f32,
    y: f32,
    z: f32,
    cube_events: &mut ResMut<Events<CubePositionEvent>>,
) {
    cube_events.send(CubePositionEvent(x, y, z));
}

fn handle_create_message_event(
    msg: String,
    conn: Connection,
    ci: &Res<ConnectionInfo>,
    client_update_events: &mut ResMut<Events<ClientUpdateEvent>>,
    players: &mut ResMut<Players>,
) {
    if ci.is_server() {
        if let Some(name) = players.names.get(&conn) {
            client_update_events.send(ClientUpdateEvent {
                from: name.to_string(),
                update: msg,
            })
        }
    }
}

fn handle_sync_messages_event(
    messages: Vec<Message>,
    ci: &Res<ConnectionInfo>,
    sync_messages_events: &mut ResMut<Events<SyncMessagesEvent>>,
) {
    if ci.is_client() {
        sync_messages_events.send(SyncMessagesEvent { messages })
    }
}

fn broadcast_all_messages(net: Res<NetworkResource>, mut messages_query: Query<&Message>) {
    let mut messages = Vec::new();
    for msg in &mut messages_query.iter() {
        messages.push(msg.clone());
    }

    let sync_messages = TestbedMessage::SyncMessages { messages };

    let _ = net.broadcast(
        &sync_messages.encode()[..],
        NetworkDelivery::ReliableSequenced(Some(1)),
    );
}

fn start_server(addr: SocketAddr, mut net: ResMut<NetworkResource>) {
    net.bind(addr).expect("We failed to bind to the socket.");
}

fn start_client(
    name: &str,
    addr: SocketAddr,
    server_addr: SocketAddr,
    mut net: ResMut<NetworkResource>,
) {
    net.bind(addr).expect("We failed to bind to the socket.");

    net.send(
        server_addr,
        &TestbedMessage::Introduction(name.to_string()).encode()[..],
        NetworkDelivery::ReliableSequenced(Some(1)),
    )
    .expect("We failed to send our introduction message");
}

impl TestbedMessage {
    pub fn encode(&self) -> Vec<u8> {
        if SERIALIZE_JSON {
            let encoded_json = serde_json::to_string(&self).unwrap();
            let bytes: Vec<u8> = encoded_json.as_bytes().to_vec();
            bytes
        } else {
            bincode::serialize(&self).unwrap()
        }
    }

    pub fn decode(bytes: &[u8]) -> TestbedMessage {
        if SERIALIZE_JSON {
            let encoded_json = std::str::from_utf8(bytes).unwrap();
            serde_json::from_str(&encoded_json).unwrap()
        } else {
            bincode::deserialize(bytes).unwrap()
        }
    }
}
