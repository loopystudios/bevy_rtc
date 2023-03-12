use futures::{select, FutureExt};
use futures_timer::Delay;
use log::{info, warn};
use matchbox_socket::PeerState;
use silk_common::{SilkSocket, SilkSocketConfig};
use silk_server::state::ServerState;
use std::time::Duration;

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

async fn async_main() {
    info!("Connecting to matchbox");
    let mut server_state = ServerState::default();
    let config = SilkSocketConfig::LocalSignallerAsHost { port: 3536 };
    let socket = SilkSocket::new(config);
    let (mut socket, loop_fut) = socket.into_parts();

    let loop_fut = loop_fut.fuse();
    futures::pin_mut!(loop_fut);

    let timeout = Delay::new(Duration::from_millis(100));
    futures::pin_mut!(timeout);

    let broadcast_every = Delay::new(Duration::from_millis(5000));
    futures::pin_mut!(broadcast_every);

    'tick: loop {
        for (peer, state) in socket.update_peers() {
            match state {
                PeerState::Connected => {
                    info!("Found a peer {:?}", peer);
                    let packet =
                        "hello client!".as_bytes().to_vec().into_boxed_slice();
                    socket.send_on_channel(
                        packet,
                        peer.clone(),
                        SilkSocketConfig::RELIABLE_CHANNEL_INDEX,
                    );
                    server_state.clients.insert(peer);
                }
                PeerState::Disconnected => {
                    info!("Disconnected peer: {:?}", peer);
                    server_state.clients.remove(&peer);
                }
            }
        }

        // Check for new messages
        for (peer, packet) in
            socket.receive_on_channel(SilkSocketConfig::RELIABLE_CHANNEL_INDEX)
        {
            info!(
                "Received from {:?}: {:?}",
                peer,
                String::from_utf8_lossy(&packet)
            );
            for client in server_state.clients.iter() {
                if *client != peer {
                    info!("forwarding to {client}");
                    socket.send_on_channel(
                        packet.clone(),
                        client,
                        SilkSocketConfig::RELIABLE_CHANNEL_INDEX,
                    );
                }
            }
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
                    socket.send_on_channel(packet, client, SilkSocketConfig::RELIABLE_CHANNEL_INDEX);
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
