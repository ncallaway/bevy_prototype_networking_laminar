## Testbed

This project current acts as a testbed for the coming laminar networking prototype. To run as a server run `cargo run -- -s`. To run as a client run `cargo run -- -c <client-name>`.

### Server

When on the server, you:

- can control the position of the cube with `WASD`
- can click the "send a message" button to add random message to the `MESSAGES` list

### Client

When on the client, you:

- cannot control the position of the cube
- (Not Implemented) the cube's position should be syncrhonized with the server
- (Not Implemented) you can click the "send a message" button to add a random message to the `MESSAGES` list
- (Not Implemented) the `MESSAGES` list is syncrhonized with the server.
