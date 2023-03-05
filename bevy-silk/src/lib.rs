use std::net::IpAddr;

use bevy::{prelude::*, tasks::IoTaskPool};
use events::SilkSocketEvent;
use matchbox_socket::WebRtcSocket;
use silk_common::{SilkSocket, SilkSocketConfig};
pub mod events;

pub struct SilkClientPlugin;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
}

impl Plugin for SilkClientPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SocketResource::default())
            .add_state(ConnectionState::Disconnected)
            .add_event::<ConnectToRemoteHostEvent>()
            .add_system(event_reader)
            .add_event::<SilkSocketEvent>()
            .add_system(event_writer)
            .add_system_set(
                SystemSet::on_enter(ConnectionState::Connecting)
                    .with_system(init_socket),
            );
    }
}

#[derive(Resource, Default)]
struct SocketResource {
    // The ID the signalling server sees us as
    pub id: Option<String>,
    // The silk socket configuration, used for connecting/reconnecting
    pub silk_config: Option<SilkSocketConfig>,
    // The underlying matchbox socket being translated
    pub mb_socket: Option<WebRtcSocket>,
}

pub struct ConnectToRemoteHostEvent {
    pub ip: IpAddr,
    pub port: u16,
}

fn event_reader(
    mut event_reader: EventReader<ConnectToRemoteHostEvent>,
    mut socket_res: ResMut<SocketResource>,
    mut connection_state: ResMut<State<ConnectionState>>,
) {
    // If disconnected, check if we were requested to reconnect
    if let ConnectionState::Disconnected = connection_state.current() {
        if let Some(ev) = event_reader.iter().next() {
            let silk_socket_config = SilkSocketConfig::RemoteSignallerClient {
                ip: ev.ip,
                port: ev.port,
            };

            info!("set state to Connecting");
            socket_res.silk_config = Some(silk_socket_config);
            info!("prev state: {:?}", connection_state);
            connection_state
                .overwrite_set(ConnectionState::Connecting)
                .unwrap();
        }
        event_reader.clear();
    }
}

// Init socket when connecting or reconnecting (on entering
// ConnectionState::Connecting)
fn init_socket(mut socket_res: ResMut<SocketResource>) {
    if let Some(silk_socket_config) = &socket_res.silk_config {
        debug!("silk config: {silk_socket_config:?}");

        // Crease silk socket
        let silk_socket = SilkSocket::new(silk_socket_config.clone());
        // Translate to matchbox parts
        let (socket, loop_fut) = silk_socket.into_parts();

        // The loop_fut runs the socket, and is async, so we use Bevy's polling.
        let task_pool = IoTaskPool::get();
        task_pool.spawn(loop_fut).detach();

        socket_res.mb_socket.replace(socket);
    } else {
        panic!("state set to connecting without config");
    }
}

fn event_writer(
    mut socket_res: ResMut<SocketResource>,
    mut event_wtr: EventWriter<SilkSocketEvent>,
    mut connection_state: ResMut<State<ConnectionState>>,
) {
    let socket_res = socket_res.as_mut();
    if let Some(ref mut socket) = socket_res.mb_socket {
        // Create socket events for Silk

        // Connection state updates
        for (id, state) in socket.update_peers() {
            match state {
                matchbox_socket::PeerState::Connected => {
                    connection_state.set(ConnectionState::Connected).unwrap();
                    event_wtr.send(SilkSocketEvent::ConnectedToHost(id));
                }
                matchbox_socket::PeerState::Disconnected => {
                    connection_state
                        .set(ConnectionState::Disconnected)
                        .unwrap();
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
