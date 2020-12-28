use crossbeam_channel::{unbounded, Receiver, Sender};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use bytes::Bytes;

use laminar::{Packet, Socket, SocketEvent};

use super::error::NetworkError;
use super::{Connection, Message, NetworkEvent, NetworkResource, SocketHandle, WorkerInstructions};

const SEND_EXPECT: &str =
    "The networking worker thread is no longer able to send messages back to the receiver.";

pub fn start_worker_thread() -> NetworkResource {
    let (event_tx, event_rx): (Sender<NetworkEvent>, Receiver<NetworkEvent>) = unbounded();
    let (message_tx, message_rx): (Sender<Message>, Receiver<Message>) = unbounded();
    let (instruction_tx, instruction_rx): (
        Sender<WorkerInstructions>,
        Receiver<WorkerInstructions>,
    ) = unbounded();

    let mut sockets = TrackedSockets {
        sockets: Vec::new(),
    };

    let sleep_time = Duration::from_millis(1);

    let resource = NetworkResource {
        default_socket: None,
        bound_sockets: Vec::new(),
        connections: Vec::new(),
        message_tx: Mutex::new(message_tx),
        event_rx: Mutex::new(event_rx),
        instruction_tx: Mutex::new(instruction_tx),
    };

    let mut start = std::time::Instant::now();
    let mut end = std::time::Instant::now();

    thread::spawn(move || loop {
        let millis = start.elapsed().as_millis();
        if millis > 50 {
            println!(
                "warning: thread worker loop took {:.3?} ({:.3?} after sleeping)",
                start.elapsed(),
                end.elapsed()
            );
        }

        start = std::time::Instant::now();

        let should_terminate = handle_instructions(&mut sockets, &instruction_rx);
        if should_terminate {
            break;
        }
        poll_sockets(&mut sockets);
        send_messages(&mut sockets, &message_rx, &event_tx);
        receive_messages(&mut sockets, &event_tx);

        end = std::time::Instant::now();

        // go dark
        std::thread::sleep(sleep_time);
    });

    resource
}

fn handle_instructions(
    sockets: &mut TrackedSockets,
    instruction_rx: &Receiver<WorkerInstructions>,
) -> bool {
    while let Ok(instruction) = instruction_rx.try_recv() {
        match instruction {
            WorkerInstructions::AddSocket(handle, socket) => {
                sockets.add_socket(handle, socket);
            } // future work: allow manual closing of sockets
            // WorkerInstructions::CloseSocket(handle) => {
            //     sockets.close_socket(handle);
            // }
            WorkerInstructions::Terminate => return true,
        }
    }

    false
}

fn poll_sockets(sockets: &mut TrackedSockets) {
    for (_, socket) in sockets.iter_mut() {
        socket.manual_poll(Instant::now());
    }
}

fn send_messages(
    sockets: &mut TrackedSockets,
    message_rx: &Receiver<Message>,
    event_tx: &Sender<NetworkEvent>,
) {
    while let Ok(message) = message_rx.try_recv() {
        let handle = message.socket_handle;

        sockets
            .get_socket_mut(handle)
            .and_then(|socket| {
                socket
                    .send(Packet::reliable_unordered(
                        message.destination,
                        message.message.to_vec(),
                    ))
                    .map_err(|e| e.into())
            })
            .or_else(|err| event_tx.send(NetworkEvent::SendError(err)))
            // this expect() is OK, since our only way of communicating errors back to the callers through this event channel. If
            // we can no longer push events back through this channel, it's time to panic.
            .expect(SEND_EXPECT);
    }
}

fn receive_messages(sockets: &mut TrackedSockets, event_tx: &Sender<NetworkEvent>) {
    for (socket_handle, socket) in sockets.iter_mut() {
        while let Some(event) = socket.recv() {
            let e = match event {
                SocketEvent::Connect(addr) => Some(NetworkEvent::Connected(Connection {
                    addr,
                    socket: *socket_handle,
                })),
                SocketEvent::Timeout(addr) | SocketEvent::Disconnect(addr) => {
                    Some(NetworkEvent::Disconnected(Connection {
                        addr,
                        socket: *socket_handle,
                    }))
                }
                SocketEvent::Packet(packet) => Some(NetworkEvent::Message(
                    Connection {
                        addr: packet.addr(),
                        socket: *socket_handle,
                    },
                    Bytes::copy_from_slice(packet.payload()),
                )),
            };

            if let Some(e) = e {
                // this expect() is OK, since our only way of communicating errors back to the callers through this event channel. If
                // we can no longer push events back through this channel, it's time to panic.
                event_tx.send(e).expect(SEND_EXPECT);
            }
        }
    }
}

struct TrackedSockets {
    sockets: Vec<(SocketHandle, Socket)>,
}

impl TrackedSockets {
    pub fn iter_mut(&mut self) -> std::slice::IterMut<(SocketHandle, Socket)> {
        self.sockets.iter_mut()
    }

    pub fn add_socket(&mut self, handle: SocketHandle, socket: Socket) {
        if self.has_socket(handle) {
            // todo: communicate socket error back
            println!(
                "Warning: attempted to add socket with an existing handle, dropping the new socket"
            );
            return;
        }

        self.sockets.push((handle, socket));
    }

    // pub fn close_socket(&mut self, handle: SocketHandle) {
    //     let sock = self.sockets.iter().position(|(h, _)| *h == handle);

    //     match sock {
    //         Some(idx) => {
    //             self.sockets.remove(idx);
    //         }
    //         None => {
    //             println!("Warning: attempting to close a socket that doesn't exist.");
    //         }
    //     }
    // }

    pub fn has_socket(&self, handle: SocketHandle) -> bool {
        self.get_socket(handle).is_ok()
    }

    pub fn get_socket(&self, handle: SocketHandle) -> Result<&Socket, NetworkError> {
        self.sockets
            .iter()
            .find(|(h, _)| handle == *h)
            .map(|(_, s)| s)
            .ok_or(NetworkError::NoSocket(handle))
    }

    pub fn get_socket_mut(&mut self, handle: SocketHandle) -> Result<&mut Socket, NetworkError> {
        self.sockets
            .iter_mut()
            .find(|(h, _)| handle == *h)
            .map(|(_, s)| s)
            .ok_or(NetworkError::NoSocket(handle))
    }
}
