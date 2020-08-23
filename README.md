# Bevy Prototype Laminar Networking Plugin

[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)

**Warning: This is a prototype and not ready for production use**

This is a prototype of a networking crate for [`bevy`](https://github.com/bevyengine/bevy). This create provides a low-level networking plugin built on top of [`laminar`](https://github.com/amethyst/laminar), which adds some simple reliability, ordering, and virtual connection options on top of a UDP socket.

## Getting Started

## Examples

### Testbed

The testbed is a simple project that provides a more comprehensive example of using `bevy_prototype_laminar_networking`.

![Testbed Screenshot](assets/screenshots/testbed-screenshot.png)

The testbed is also is intended to serve as a testbed for any other networking prototypes or attempts. All interaction with `bevy_prototype_laminar_networking` is contained to `examples/testbed/net/prototype.rs`. Using the testbed with a different networking plugin should be as simple as updating `prototype.rs` to interact with the other networking plugin. Contributions to the testbed to improve the code quality, or make the testbed more comprehensive by adding other prototypical network interactions are welcome.

- `cargo run --example testbed -- -s 127.0.0.1:12540` to start a server
- `cargo run --example testbed -- -c 127.0.0.1:12541 127.0.0.1:12540 foo` to start a client named `foo` connecting to the server

#### Server

When on the server, you:

- can control the position of the cube with `WASD`
- can click the "send a message" button to add random message to the `MESSAGES` list

#### Client

When on the client, you:

- cannot control the position of the cube
- the cube's position should be syncrhonized with the server
- you can click the "send a message" button to add a random message to the `MESSAGES` list
- the `MESSAGES` list is syncrhonized with the server.

### simple

The simple example shows a very bare bones `bevy` application that will send messages back and forth.

- `cargo run --example simple -- -s` start a server
- `cargo run --example simple -- -c` start a client

```
Network Event: Message(Connection { addr: V4(127.0.0.1:12351), socket: SocketHandle { identifier: 0 } }, b"How are things over there?")
Network Event: Connected(Connection { addr: V4(127.0.0.1:12351), socket: SocketHandle { identifier: 0 } })
Network Event: Disconnected(Connection { addr: V4(127.0.0.1:12351), socket: SocketHandle { identifier: 0 } })
Network Event: Message(Connection { addr: V4(127.0.0.1:12351), socket: SocketHandle { identifier: 0 } }, b"How are things over there?")
Network Event: Connected(Connection { addr: V4(127.0.0.1:12351), socket: SocketHandle { identifier: 0 } })
Network Event: Disconnected(Connection { addr: V4(127.0.0.1:12351), socket: SocketHandle { identifier: 0 } })
```

## Future Work

## License

Licesened under the [MIT license](https://opensource.org/licenses/MIT).
