use bevy::{prelude::*, tasks::IoTaskPool};
use events::SilkSocketEvent;
use matchbox_socket::{WebRtcSocket, WebRtcSocketConfig};
use silk_common::SilkSocketConfig;
pub mod events;

pub struct SilkClientPlugin {
    pub config: SilkSocketConfig,
}

impl Plugin for SilkClientPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SocketResource {
            id: None,
            config: self.config.get(),
            socket: None,
        })
        .add_startup_system(init_socket)
        .add_event::<SilkSocketEvent>()
        .add_system(event_writer);
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

fn event_writer(
    mut socket_res: ResMut<SocketResource>,
    mut event_wtr: EventWriter<SilkSocketEvent>,
) {
    let socket_res = socket_res.as_mut();
    if let Some(ref mut socket) = socket_res.socket {
        // Create socket events for Silk

        // Connection state updates
        for (id, state) in socket.update_peers() {
            match state {
                matchbox_socket::PeerState::Connected => {
                    event_wtr.send(SilkSocketEvent::ConnectedToHost(id));
                }
                matchbox_socket::PeerState::Disconnected => {
                    event_wtr.send(SilkSocketEvent::DisconnectedFromHost(id));
                }
            }
        }

        // Unreliable messages
        event_wtr.send_batch(
            socket
                .receive_on_channel(SilkSocketConfig::UNRELIABLE_CHANNEL_INDEX)
                .into_iter()
                .map(SilkSocketEvent::Message),
        );

        // Reliable messages
        event_wtr.send_batch(
            socket
                .receive_on_channel(SilkSocketConfig::RELIABLE_CHANNEL_INDEX)
                .into_iter()
                .map(SilkSocketEvent::Message),
        );

        // Id changed events
        match socket.id() {
            Some(id) => {
                if socket_res.id.is_none() {
                    socket_res.id.replace(id.clone());
                    event_wtr.send(SilkSocketEvent::IdAssigned(id));
                }
            }
            None => {
                if socket_res.id.is_some() {
                    socket_res.id.as_mut().take();
                    event_wtr.send(SilkSocketEvent::IdRemoved);
                }
            }
        }
    }
}
