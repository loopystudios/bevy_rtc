#![feature(async_closure)]
use futures::{select, FutureExt};
use futures_timer::Delay;
use log::{debug, error, info};
use matchbox_socket::{PeerState, WebRtcSocket};
use silk_common::SilkSocketConfig;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
    },
    thread,
    time::Duration,
};

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
                    "silk_client=debug,matchbox_socket=info".into()
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

async fn async_main() {
    info!("Connecting to matchbox");
    let config = SilkSocketConfig::LocalClient { port: 3536 }.get();
    let (mut socket, loop_fut) = WebRtcSocket::new_with_config(config);

    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
    thread::spawn({
        let tx = tx.clone();
        move || loop {
            let mut buffer = String::new();
            std::io::stdin().read_line(&mut buffer).unwrap();
            tx.send(buffer).unwrap();
        }
    });

    let connected = AtomicBool::new(false);
    let mut host_peer_id = None;

    let loop_fut = loop_fut.fuse();
    futures::pin_mut!(loop_fut);

    let timeout = Delay::new(Duration::from_millis(100));
    futures::pin_mut!(timeout);

    'client: loop {
        for (peer, state) in socket.update_peers() {
            match state {
                PeerState::Connected => {
                    if !connected.load(Ordering::Acquire) {
                        let packet = "hello server!"
                            .as_bytes()
                            .to_vec()
                            .into_boxed_slice();
                        socket.send(packet, peer.clone());
                        host_peer_id.replace(peer);
                        connected.store(true, Ordering::Release);
                    } else {
                        error!("socket already connected to a host");
                    }
                }
                PeerState::Disconnected => {
                    if connected.load(Ordering::Acquire) {
                        info!("Host disconnected!");
                        connected.store(false, Ordering::Release);
                        host_peer_id.take();
                        break 'client;
                    }
                }
            }
        }

        for (peer, packet) in
            socket.receive_on_channel(SilkSocketConfig::RELIABLE_CHANNEL_INDEX)
        {
            info!(
                "Received from {:?}: {:?}",
                peer,
                String::from_utf8_lossy(&packet)
            );
        }

        if connected.load(Ordering::Relaxed) {
            while let Ok(line) = rx.try_recv() {
                debug!("sending: {line}");
                let packet = line.as_bytes().to_vec().into_boxed_slice();
                let host_peer_id = host_peer_id.as_ref().unwrap();
                socket.send_on_channel(
                    packet,
                    host_peer_id,
                    SilkSocketConfig::RELIABLE_CHANNEL_INDEX,
                );
            }
        }

        select! {
            _ = (&mut timeout).fuse() => {
                timeout.reset(Duration::from_millis(100));
            }

            _ = &mut loop_fut => {
                break 'client;
            }
        }
    }
}
