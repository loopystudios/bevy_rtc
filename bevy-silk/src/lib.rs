use std::net::IpAddr;

use bevy::{prelude::*, tasks::IoTaskPool};
use events::SocketEvent;
use matchbox_socket::{WebRtcSocket, WebRtcSocketConfig};
use silk_common::SocketConfig;
mod events;

pub struct SilkClientPlugin {
    config: WebRtcSocketConfig,
}

impl SilkClientPlugin {
    fn new(silk_config: SocketConfig) -> Self {
        Self {
            config: silk_config.get(),
        }
    }
}

impl Plugin for SilkClientPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SocketResource {
            id: None,
            config: self.config.clone(),
            socket: None,
        })
        .add_startup_system(init_socket)
        .add_system(poll_socket);
    }
}

#[derive(Resource)]
struct SocketResource {
    pub id: Option<String>,
    pub config: WebRtcSocketConfig,
    pub socket: Option<WebRtcSocket>,
}

fn init_socket(mut socket_res: ResMut<SocketResource>) {
    debug!("socket config: {:?}", socket_res.as_ref().config);
    info!("connecting to {}...", socket_res.as_ref().config.room_url);

    let (socket, loop_fut) =
        WebRtcSocket::new_with_config(socket_res.as_mut().config.clone());

    // The loop_fut runs the socket, and is async, so we use Bevy's polling.
    let task_pool = IoTaskPool::get();
    task_pool.spawn(loop_fut).detach();

    socket_res.as_mut().socket.replace(socket);
}

fn poll_socket(
    mut socket_res: ResMut<SocketResource>,
    mut event_wtr: EventWriter<SocketEvent>,
) {
    let socket_res = socket_res.as_mut();
    if let Some(ref mut socket) = socket_res.socket {
        // Forward socket events unadulterated as events to Bevy.

        // Peer connection state updates
        event_wtr.send_batch(
            socket
                .update_peers()
                .into_iter()
                .map(SocketEvent::PeerStateChange),
        );

        // Unreliable messages
        event_wtr.send_batch(
            socket
                .receive_on_channel(SocketConfig::UNRELIABLE_CHANNEL_INDEX)
                .into_iter()
                .map(SocketEvent::Message),
        );

        // Reliable messages
        event_wtr.send_batch(
            socket
                .receive_on_channel(SocketConfig::RELIABLE_CHANNEL_INDEX)
                .into_iter()
                .map(SocketEvent::Message),
        );

        // Id changed events
        match socket.id() {
            Some(id) => {
                if socket_res.id.is_none() {
                    socket_res.id.replace(id.clone());
                    event_wtr.send(SocketEvent::IdAssigned(id));
                }
            }
            None => {
                if socket_res.id.is_some() {
                    socket_res.id.as_mut().take();
                    event_wtr.send(SocketEvent::IdRemoved);
                }
            }
        }
    }
}
