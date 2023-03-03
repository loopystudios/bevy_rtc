use futures::{select, FutureExt};
use futures_timer::Delay;
use log::{error, info};
use matchbox_socket::{PeerState, WebRtcSocket};
use std::{
    sync::atomic::{AtomicBool, Ordering},
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
                .unwrap_or_else(|_| "info,matchbox_socket=info".into()),
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
    let (mut socket, loop_fut) =
        WebRtcSocket::new_unreliable("ws://localhost:3536/Client");

    let connected = AtomicBool::new(false);
    info!("my id is {:?}", socket.id());

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
                        socket.send(packet, peer);
                        connected.store(true, Ordering::Release);
                    } else {
                        error!("socket already connected to a host");
                    }
                }
                PeerState::Disconnected => {
                    if connected.load(Ordering::Acquire) {
                        info!("Host disconnected!");
                        connected.store(false, Ordering::Release);
                        break 'client;
                    }
                }
            }
        }

        for (peer, packet) in socket.receive() {
            info!(
                "Received from {:?}: {:?}",
                peer,
                String::from_utf8_lossy(&packet)
            );
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
