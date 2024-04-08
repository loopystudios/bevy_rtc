# bevy_rtc

![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)
[![crates.io](https://img.shields.io/crates/v/bevy_rtc.svg)](https://crates.io/crates/bevy_rtc)
[![docs.rs](https://img.shields.io/docsrs/bevy_rtc)](https://docs.rs/bevy_rtc)

bevy_rtc is a simple, multi-platform WebRTC networking library for client<->server topologies using Bevy.

- Simple: no knowledge of WebRTC is needed
- Easy unreliable (UDP-like) and reliable (TCP-like) networking on web
- Bevy system parameters for reading and writing packets
- Derive macros for creating protocols
- Support for unbounded and bounded buffers
- Easily read instantaneous and smoothed latency

## Quickstart

For your client:

```shell
cargo add bevy_rtc -F client
```

For your server:

```shell
cargo add bevy_rtc -F server
```

Run the [demos](#demos) and [instructions](#instructions).

## Bevy version support

| bevy  |  bevy_rtc   |
|-------|-------------|
| 0.13  | 0.1-0.2, main   |
| < 0.13| unsupported |

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

## Demos

There are two demos provided, a simple ping/pong demo and a painting game.
Run one demo server, and any number of respective clients.

- Server (Native only)

```bash
cargo run -p painting-server
```

- Client (Native)

```bash
cargo run -p painting-client
```

- Client (Web)

```bash
cargo install wasm-server-runner
cd demos/painting-client
cargo run --target wasm32-unknown-unknown
```

## Instructions

### Protocols

Place your packet definitions in a shared location.

```rust
#[derive(Payload)]
pub enum MyPacket {
    Ping,
    Pong
}
```

  **Need help?** See the [demo protocol](demos/protocol/) source or [open an issue](/issues).

### Server

- Ensure your client has the `server` feature

  ```shell
  cargo add bevy_rtc -F server
  ```

- Add the `RtcServerPlugin` to your app.

  ```rust
  .add_plugins(RtcServerPlugin { port: 3536 })
  ```

- Register your protocols as bounded or unbounded.
  - Bounded protocols will only keep the most recent N payloads received to read.
  - Unbounded protocols will keep all payloads using a resizable buffer.

  Payloads are only flushed by a system when they get read! \
  **It is recommended to keep your protocols _bounded_ on the server**.

  ```rust
  // Only choose one!
  .add_bounded_protocol::<MyPacket>(5) // Only keep the most recent 5 payloads for reading
  .add_unbounded_protocol::<MyPacket>() // Keep all payloads until read
  ```

- Add systems to read and send payloads.

    ```rust
    .add_systems(
        Update,
        |mut server: RtcServer<MyPacket>| {
            for (peer_id, packet) in server.read() {
                if let MyPacket::Ping = packet {
                    server.reliable_to_peer(peer_id, MyPacket::Pong);
                }
            }
        })
    ```

  **Need help?** See the [ping-server](demos/ping-server/) or [painting-server](demos/painting-server/) source or [open an issue](/issues).

### Client

- Ensure your client has the `client` feature

  ```shell
  cargo add bevy_rtc -F client
  ```

- Add the `RtcClientPlugin` to your app.

  ```rust
  .add_plugins(RtcClientPlugin)
  ```

- Register your protocols as bounded or unbounded.
  - Bounded protocols will only keep the most recent N payloads received to read.
  - Unbounded protocols will keep all payloads using a resizable buffer.

  Payloads are only flushed by a system when they get read! \
  **It is recommended to keep your protocols _unbounded_ on the client**.

  ```rust
  // Only choose one!
  .add_bounded_protocol::<MyPacket>(5) // Only keep the most recent 5 payloads for reading
  .add_unbounded_protocol::<MyPacket>() // Keep all payloads until read
  ```

- Add systems to read and send payloads.

    ```rust
    .add_systems(
        Update,
        {
            |mut client: RtcClient<PingPayload>| {
                client.reliable_to_host(PingPayload::Ping);
            }
        }
        .run_if(
            // Only send every second, and if we are connected.
            on_timer(Duration::from_secs(1)).and_then(
                state_exists_and_equals(RtcClientStatus::Connected),
            ),
        ),
    )
    .add_systems(Update, |mut client: RtcClient<PingPayload>| {
        for payload in client.read() {
            if let PingPayload::Pong = payload {
                info!("..Received pong!");
            }
        }
    })
    ```

  **Need help?** See the [ping-client](demos/ping-client/) or [ping-server](demos/ping-server/) source or [open an issue](/issues).

## Community

All Loopy projects and development happens in the [Loopy Discord](https://discord.gg/zrjnQzdjCB). The discord is open to the public.

Contributions are welcome by pull request. The [Rust code of conduct](https://www.rust-lang.org/policies/code-of-conduct) applies.

## License

Licensed under either of

- Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
