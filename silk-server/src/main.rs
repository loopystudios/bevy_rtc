use futures::{select, FutureExt};
use futures_timer::Delay;
use log::{info, warn};
use matchbox_socket::WebRtcSocket;
use std::{collections::HashSet, time::Duration};

#[tokio::main]
async fn main() {
    use tracing_subscriber::prelude::*;
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,matchbox_socket=debug".into()),
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

    let loop_fut = loop_fut.fuse();
    futures::pin_mut!(loop_fut);

    let timeout = Delay::new(Duration::from_millis(100));
    futures::pin_mut!(timeout);

    let broadcast_every = Delay::new(Duration::from_millis(5000));
    futures::pin_mut!(broadcast_every);

    'tick: loop {
        // Check for new peers
        for peer in socket.accept_new_connections() {
            info!("Found a peer {:?}", peer);
            let packet = "hello client!".as_bytes().to_vec().into_boxed_slice();
            socket.send(packet, peer.clone());
            server_state.clients.insert(peer);
        }
        // Check for peer disconnects
        for peer in socket.disconnected_peers() {
            info!("Disconnected peer: {:?}", peer);
            server_state.clients.remove(&peer);
        }

        // Check for new messages
        for (peer, packet) in socket.receive() {
            info!(
                "Received from {:?}: {:?}",
                peer,
                String::from_utf8_lossy(&packet)
            );
        }

        select! {
            // Restart the tick loop every 100ms
            _ = (&mut timeout).fuse() => {
                timeout.reset(Duration::from_millis(100));
            }

            // Every 5s, send messages to everyone with personally identifying information
            _ = (&mut broadcast_every).fuse() => {
                for client in server_state.clients.iter() {
                    let packet = format!("Hello {}, the server has {} clients", client, server_state.clients.len())
                        .as_bytes().to_vec().into_boxed_slice();
                    socket.send(packet, client);
                }
                broadcast_every.reset(Duration::from_millis(5000));
            }

            // Check if the signalling server connection cut
            _ = &mut loop_fut => {
                warn!("Connect ended");
                break 'tick;
            }
        }
    }
}
