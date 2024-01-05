# bevy-silk

bevy-silk is a simple, multi-platform WebRTC networking library for client<->server topologies using Bevy. No knowledge of WebRTC is required.

> [!IMPORTANT]
>
> - The `client` feature supports **WASM and native** targets.
> - The `server` feature is **native only**.
>
> The server is native only because it serves a signaling server (and is the first peer of itself). While a WASM server is possible by depending on an external WebRTC signaling server, WASM is (currently) single threaded like JavaScript. If you really want a WASM server, I would accept PRs, but you probably don't want one!

## Compatibility

| bevy  | bevy_matchbox |  bevy-silk  |
|-------|---------------|-------------|
| 0.12  | 0.8           | 0.8, main   |
| 0.11  | 0.7, main     | 0.7         |
| 0.10  | 0.6           | unsupported |
| < 0.9 | unsupported   | unsupported |

## Features

All features are opt-in.

- `server` - Provides networking for server applications
- `client` - Provides networking for client applications
- `binary` - Sends networking packets as binary instead of JSON (the default)

## Demo

Run the server, and any number of clients.

- Server (Native only)

```bash
cargo run -p server
```

- Client (Native)

```bash
cargo run -p client
```

- Client (Web)

```bash
cargo install wasm-server-runner
cargo run -p client --target wasm32-unknown-unknown
```

## Example

Place your packet definitions in a shared location. The demo does this in a shared crate called [`protocol`](demo/protocol/).

```rust
#[derive(Payload)]
pub enum MyPacket {
    Ping,
    Pong
}
```

Server:

```rust
use protocol::MyPacket;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(SilkServerPlugin {
            signaling_port: 3536,
            tick_rate: 1.0, // listen/respond to packets once a second
        })
        .add_network_message::<MyPacket>()
        .add_systems(
            SilkSchedule,
            |reader: NetworkReader<MyPacket>, writer: NetworkWriter<MyPacket>| {
                for (peer_id, packet) in reader.read() {
                    if let MyPacket::Ping = packet {
                        println!("Received ping, sending pong!");
                        writer.send_reliable_to(peer_id, MyPacket::Pong);
                    }
                }
            }
            .in_set(SilkSet::Update), // Optional: When to run your system
        )
        .add_systems(Startup, || println!("Started server on ws://0.0.0.0:3536"))
        .run();
}
```

Client:

```rust
use protocol::MyPacket;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(SilkClientPlugin)
        .add_network_message::<Chat>()
        .add_systems(
            SilkSchedule,
            |reader: NetworkReader<MyPacket>| {
                for (peer_id, packet) in reader.read() {
                    if let MyPacket::Pong = packet {
                        println!("Received pong!");
                    }
                }
            }
        )
        .add_systems(
            Startup,
            |writer: NetworkWriter<MyPacket>| {
                println!("Sent ping!");
                writer.reliable_to_host(MyPacket::Ping);
            }
        )
        .run();
}
```

## Notes

- Any network traffic (reading, writing) must be scheduled on the `SilkSchedule`.
- Do not put rendering or UI on the `SilkSchedule`, or you may see frame dropping. Instead, use shared memory for rendering (e.g. resources, entities, etc.).
