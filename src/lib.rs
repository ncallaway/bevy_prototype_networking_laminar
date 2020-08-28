use bevy::prelude::*;

use crossbeam_channel::{Receiver, Sender};
use std::fmt;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Mutex;

use laminar::Socket;

use bytes::Bytes;
use uuid::Uuid;

mod error;
mod transport;
mod worker;

pub use error::NetworkError;
pub use transport::{LaminarConfig, Transport};

pub struct NetworkingPlugin;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SocketHandle(uuid::Uuid);

impl SocketHandle {
    fn new() -> Self {
        // We're using UUID here to mirror the way bevy currently treats asset handles. Since sockets handles are specific to a single process, and it's
        // unlikely anyone will have a large number of sockets, we could switching to a u32.
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Connection {
    pub addr: SocketAddr,
    pub socket: SocketHandle,
}

impl fmt::Display for Connection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.addr)
    }
}

#[derive(Debug)]
pub enum NetworkEvent {
    Connected(Connection),
    Disconnected(Connection),
    Message(Connection, Bytes),
    SendError(NetworkError),
}
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum NetworkDelivery {
    UnreliableUnordered,
    UnreliableSequenced(Option<u8>),
    ReliableUnordered,
    ReliableSequenced(Option<u8>),
    ReliableOrdered(Option<u8>),
}

pub struct NetworkResource {
    default_socket: Option<SocketHandle>,

    bound_sockets: Vec<SocketHandle>,
    connections: Vec<Connection>,
    event_rx: Mutex<Receiver<NetworkEvent>>,
    message_tx: Mutex<Sender<Message>>,
    instruction_tx: Mutex<Sender<WorkerInstructions>>,
}

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        let network_resource = worker::start_worker_thread();

        app.add_event::<NetworkEvent>()
            .add_resource(network_resource)
            .add_system(process_network_events.system());
    }
}

impl NetworkResource {
    pub fn connections(&self) -> &Vec<Connection> {
        &self.connections
    }

    pub fn connections_for_socket(&self, socket: SocketHandle) -> Vec<Connection> {
        self.connections
            .iter()
            .filter(|c| c.socket == socket)
            .cloned()
            .collect()
    }

    pub fn add_connection(&mut self, connection: Connection) {
        if self.has_connection(connection) {
            println!("Warning: attempted to add a connection that already exists");
            return;
        }

        self.connections.push(connection);
    }

    pub fn remove_connection(&mut self, connection: Connection) {
        let conn = self.connections.iter().position(|c| *c == connection);

        match conn {
            Some(idx) => {
                self.connections.remove(idx);
            }
            None => {
                println!("Warning: attempted to remove connection that doesn't exist");
            }
        }
    }

    pub fn has_connection(&self, connection: Connection) -> bool {
        self.connections.iter().any(|c| *c == connection)
    }

    pub fn bind<A: ToSocketAddrs>(&mut self, addr: A) -> Result<SocketHandle, NetworkError> {
        self.bind_with_transport(addr, Transport::Laminar(LaminarConfig::default()))
    }

    pub fn bind_with_transport<A: ToSocketAddrs>(
        &mut self,
        addr: A,
        transport: Transport,
    ) -> Result<SocketHandle, NetworkError> {
        match transport {
            Transport::Laminar(config) => self.bind_with_laminar(addr, config),
        }
    }

    fn bind_with_laminar<A: ToSocketAddrs>(
        &mut self,
        addr: A,
        config: LaminarConfig,
    ) -> Result<SocketHandle, NetworkError> {
        let cfg = config.into();

        let handle = SocketHandle::new();
        let socket = Socket::bind_with_config(addr, cfg)?;

        let instruction = WorkerInstructions::AddSocket(handle, socket);
        {
            let locked = self.instruction_tx.lock()?;
            locked.send(instruction)?;
        }

        self.bound_sockets.push(handle);

        if self.default_socket.is_none() {
            self.default_socket = Some(handle);
        }

        Ok(handle)
    }

    pub fn send(
        &self,
        addr: SocketAddr,
        message: &[u8],
        delivery: NetworkDelivery,
    ) -> Result<(), NetworkError> {
        self.send_with_config(addr, message, delivery, SendConfig::default())
    }

    pub fn broadcast(&self, message: &[u8], delivery: NetworkDelivery) -> Result<(), NetworkError> {
        self.broadcast_with_config(message, delivery, SendConfig::default())
    }

    pub fn send_with_config(
        &self,
        addr: SocketAddr,
        message: &[u8],
        delivery: NetworkDelivery,
        config: SendConfig,
    ) -> Result<(), NetworkError> {
        let socket = self.get_socket_or_default(config.socket)?;

        let msg = Message {
            destination: addr,
            delivery,
            socket_handle: socket,
            message: Bytes::copy_from_slice(message),
        };

        self.message_tx.lock()?.send(msg)?;

        Ok(())
    }

    pub fn broadcast_with_config(
        &self,
        message: &[u8],
        delivery: NetworkDelivery,
        config: SendConfig,
    ) -> Result<(), NetworkError> {
        let socket = self.get_socket_or_default(config.socket)?;

        let broadcast_to = self.connections_for_socket(socket);

        for conn in broadcast_to {
            let msg = Message {
                destination: conn.addr,
                delivery,
                socket_handle: socket,
                message: Bytes::copy_from_slice(message),
            };

            self.message_tx.lock()?.send(msg)?;
        }

        Ok(())
    }

    fn get_socket_or_default(
        &self,
        socket: Option<SocketHandle>,
    ) -> Result<SocketHandle, NetworkError> {
        let socket = socket
            .or(self.default_socket)
            .ok_or(NetworkError::NoDefaultSocket)?;

        match self.bound_sockets.contains(&socket) {
            true => Ok(socket),
            false => Err(NetworkError::NoSocket(socket)),
        }
    }
}

impl Drop for NetworkResource {
    fn drop(&mut self) {
        let locked = self.instruction_tx.lock().unwrap();
        locked.send(WorkerInstructions::Terminate).unwrap();
    }
}

#[derive(Default)]
pub struct SendConfig {
    pub socket: Option<SocketHandle>, // if none, use the default socket
}

#[derive(Debug)]
struct Message {
    message: Bytes,
    delivery: NetworkDelivery,
    socket_handle: SocketHandle,
    destination: SocketAddr,
}

enum WorkerInstructions {
    AddSocket(SocketHandle, Socket),
    Terminate,
}

fn process_network_events(
    mut net: ResMut<NetworkResource>,
    mut network_events: ResMut<Events<NetworkEvent>>,
) {
    let mut added_connections: Vec<Connection> = Vec::new();
    let mut removed_connections: Vec<Connection> = Vec::new();

    {
        let locked = match net.event_rx.lock() {
            Ok(l) => l,
            // this system is the only consumer of `event_rx`, so if this lock is poisoned that means
            // a previous iteration of our thread panic'd without taking down the game. We'll
            // bravely try and soldier on and continue to process network event's, but it's pretty
            // bad.
            Err(p) => p.into_inner(),
        };

        while let Ok(event) = locked.try_recv() {
            match event {
                NetworkEvent::Connected(conn) => {
                    if !net.has_connection(conn) && !added_connections.contains(&conn) {
                        added_connections.push(conn);
                    }
                }
                NetworkEvent::Disconnected(conn) => {
                    if net.has_connection(conn) && !removed_connections.contains(&conn) {
                        removed_connections.push(conn);
                    }
                }
                _ => network_events.send(event),
            }
        }
    }

    for conn in added_connections {
        net.add_connection(conn);
        network_events.send(NetworkEvent::Connected(conn));
    }

    for conn in removed_connections {
        net.remove_connection(conn);
        network_events.send(NetworkEvent::Disconnected(conn));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn network_resource_has_no_default_connections() {
        let network_resource = worker::start_worker_thread();

        assert!(network_resource.default_socket.is_none());
        assert!(network_resource.bound_sockets.is_empty());
    }

    #[test]
    fn binding_network_resource_sets_the_default_socket() {
        let mut network_resource = worker::start_worker_thread();

        assert!(network_resource.bind("127.0.0.1:12591").is_ok());
        assert!(network_resource.default_socket.is_some());
    }
}
