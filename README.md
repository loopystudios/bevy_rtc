# bevy-silk

bevy-silk is a simple, multi-platform WebRTC networking library for client<->server topologies using Bevy.

- Simple: no knowledge of WebRTC is needed
- Easy unreliable (UDP-like) and reliable (TCP-like) networking on web
- Bevy system parameters for reading and writing packets
- Derive macros for creating protocols
- Easily introspect latency

## Quickstart

For your client:

```shell
cargo add bevy-silk -F client
```

For your server:

```shell
cargo add bevy-silk -F server
```

Check out the [demo](#demo) and [example usage](#example-usage).

## Compatibility

| bevy  | bevy_matchbox |  bevy-silk  |
|-------|---------------|-------------|
| 0.12  | 0.8           | 0.8, main   |
| 0.11  | 0.7, main     | 0.7         |
| 0.10  | 0.6           | unsupported |
| < 0.9 | unsupported   | unsupported |

## Cargo features

All features are opt-in.

- `server` - Provides necessary networking for server applications
- `client` - Provides necessary networking for client applications
- `binary` - Sends networking packets as binary instead of JSON (the default)

> [!IMPORTANT]
>
> - The `client` feature supports both **WASM and native** targets.
> - The `server` feature is **native only**.
>
> The server is native only because it serves a signaling server (and is the first peer of itself). While a WASM server is possible by depending on an external WebRTC signaling server, WASM is (currently) single threaded like JavaScript. If you really want a WASM server, I would accept PRs, but you probably don't want one!

## Demo

Run the provided demo server, and any number of clients.

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

## Example Usage

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
        .add_plugins(SilkServerPlugin { port: 3536 })
        .add_network_message::<MyPacket>()
        .add_systems(
            Update,
            |reader: NetworkReader<MyPacket>, writer: NetworkWriter<MyPacket>| {
                for (peer_id, packet) in reader.read() {
                    if let MyPacket::Ping = packet {
                        println!("Received ping, sending pong!");
                        writer.send_reliable_to(peer_id, MyPacket::Pong);
                    }
                }
            }
        )
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
        .add_systems(Startup,
        |mut connection_requests: EventWriter<ConnectionRequest>| {
            connection_requests.send(ConnectionRequest::Connect {
                addr: "ws://127.0.0.1:3536".to_string()
            });
        })
        .add_systems(
            PostStartup,
            |writer: NetworkWriter<MyPacket>| {
                println!("Sent ping!");
                writer.reliable_to_host(MyPacket::Ping);
            }
        )
        .add_systems(
            Update,
            |reader: NetworkReader<MyPacket>| {
                for (peer_id, packet) in reader.read() {
                    if let MyPacket::Pong = packet {
                        println!("Received pong!");
                    }
                }
            }
        )
        .run();
}
```
