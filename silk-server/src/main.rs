use futures::{select, FutureExt};
use futures_timer::Delay;
use log::info;
use matchbox_socket::WebRtcSocket;
use std::{collections::HashSet, time::Duration};

#[cfg(target_arch = "wasm32")]
fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    wasm_bindgen_futures::spawn_local(async_main());
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    use tracing_subscriber::prelude::*;
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| {
                    "info,matchbox_simple_demo=info,matchbox_socket=info".into()
                }),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .compact()
                .with_file(false)
                .with_target(false),
        )
        .init();

    async_main().await
}

struct Clients {
    clients: HashSet<String>,
}

async fn async_main() {
    info!("Connecting to matchbox");
    let mut server_state = Clients {
        clients: HashSet::new(),
    };
    let (mut socket, loop_fut) =
        WebRtcSocket::new_unreliable("ws://localhost:3536/Host");

    info!("my server id is {:?}", socket.id());

    let loop_fut = loop_fut.fuse();
    futures::pin_mut!(loop_fut);

    let timeout = Delay::new(Duration::from_millis(100));
    futures::pin_mut!(timeout);

    let broadcast_every = Delay::new(Duration::from_millis(5000));
    futures::pin_mut!(broadcast_every);

    loop {
        for peer in socket.accept_new_connections() {
            info!("Found a peer {:?}", peer);
            let packet = "hello client!".as_bytes().to_vec().into_boxed_slice();
            socket.send(packet, peer.clone());
            server_state.clients.insert(peer);
        }

        for (peer, packet) in socket.receive() {
            info!(
                "Received from {:?}: {:?}",
                peer,
                String::from_utf8_lossy(&packet)
            );
        }
        let disconnected_peers = socket.disconnected_peers();
        if !disconnected_peers.is_empty() {
            info!("Disconnected peers: {:?}", disconnected_peers);
            for disconnected_peer in disconnected_peers {
                server_state.clients.remove(&disconnected_peer);
            }
        }

        select! {
            _ = (&mut broadcast_every).fuse() => {
                if !server_state.clients.is_empty() {
                    info!("sending propoganda");
                    for client in server_state.clients.iter() {
                        let packet = format!("Hello {}, the server has {} clients", client, server_state.clients.len())
                            .as_bytes().to_vec().into_boxed_slice();
                        socket.send(packet, client);
                    }
                }

                broadcast_every.reset(Duration::from_millis(5000));
            }

            _ = (&mut timeout).fuse() => {
                timeout.reset(Duration::from_millis(100));
            }

            _ = &mut loop_fut => {
                break;
            }
        }
    }

    info!("Done");
}
