use std::net::IpAddr;

use bevy::{prelude::*, tasks::IoTaskPool};
use events::SilkSocketEvent;
use matchbox_socket::{PeerId, WebRtcSocket};
use silk_common::{SilkSocket, SilkSocketConfig};
pub mod events;

/// The socket client abstraction
pub struct SilkClientPlugin;

/// State of the socket
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
            .add_event::<ConnectionRequest>()
            .add_system(event_reader)
            .add_event::<SilkSocketEvent>()
            .add_system(event_writer)
            .add_system_set(
                SystemSet::on_enter(ConnectionState::Connecting)
                    .with_system(init_socket),
            )
            .add_system_set(
                SystemSet::on_enter(ConnectionState::Disconnected)
                    .with_system(reset_socket),
            );
    }
}

#[derive(Resource, Default)]
struct SocketResource {
    /// The ID given by the signalling server
    pub id: Option<PeerId>,
    /// The socket configuration, used for connecting/reconnecting
    pub silk_config: Option<SilkSocketConfig>,
    /// The underlying matchbox socket
    pub mb_socket: Option<WebRtcSocket>,
}

pub enum ConnectionRequest {
    /// A request to connect to the server through the signalling server; the
    /// ip and port are the signalling server
    Connect { ip: IpAddr, port: u16 },
    /// A request to disconnect from the signalling server; this will also
    /// disconnect from the server
    Disconnect,
}

/// Initialize the socket
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

/// Reset the internal socket
fn reset_socket(mut socket_res: ResMut<SocketResource>) {
    *socket_res = SocketResource {
        id: None,
        silk_config: socket_res.silk_config.take(),
        mb_socket: None,
    };
}

/// Reads and handles connection request events
fn event_reader(
    mut cxn_event_reader: EventReader<ConnectionRequest>,
    mut socket_res: ResMut<SocketResource>,
    mut connection_state: ResMut<State<ConnectionState>>,
    mut silk_event_wtr: EventWriter<SilkSocketEvent>,
) {
    match cxn_event_reader.iter().next() {
        Some(ConnectionRequest::Connect { ip, port }) => {
            if let ConnectionState::Disconnected = connection_state.current() {
                let silk_socket_config =
                    SilkSocketConfig::RemoteSignallerClient {
                        ip: *ip,
                        port: *port,
                    };

                debug!(
                    previous = format!("{connection_state:?}"),
                    "set state: connecting"
                );
                socket_res.silk_config = Some(silk_socket_config);
                _ = connection_state.overwrite_set(ConnectionState::Connecting);
            }
        }
        Some(ConnectionRequest::Disconnect) => {
            if let ConnectionState::Connected = connection_state.current() {
                debug!(
                    previous = format!("{connection_state:?}"),
                    "set state: disconnected"
                );
                socket_res.mb_socket.take();
                silk_event_wtr.send(SilkSocketEvent::DisconnectedFromHost);
                _ = connection_state
                    .overwrite_set(ConnectionState::Disconnected);
            }
        }
        None => {}
    }
}

/// Translates socket updates into bevy events
fn event_writer(
    mut socket_res: ResMut<SocketResource>,
    mut event_wtr: EventWriter<SilkSocketEvent>,
    mut connection_state: ResMut<State<ConnectionState>>,
) {
    let socket_res = socket_res.as_mut();
    if let Some(ref mut socket) = socket_res.mb_socket {
        // Create socket events for Silk

        // Id changed events
        if let Some(id) = socket.id() {
            if socket_res.id.is_none() {
                socket_res.id.replace(id);
                event_wtr.send(SilkSocketEvent::IdAssigned(id));
            }
        }

        // Connection state updates
        for (id, state) in socket.update_peers() {
            match state {
                matchbox_socket::PeerState::Connected => {
                    _ = connection_state
                        .overwrite_set(ConnectionState::Connected);
                    event_wtr.send(SilkSocketEvent::ConnectedToHost(id));
                }
                matchbox_socket::PeerState::Disconnected => {
                    _ = connection_state
                        .overwrite_set(ConnectionState::Disconnected);
                    event_wtr.send(SilkSocketEvent::DisconnectedFromHost);
                }
            }
        }

        // Collect Unreliable, Reliable messages
        event_wtr.send_batch(
            socket
                .receive_on_channel(SilkSocketConfig::UNRELIABLE_CHANNEL_INDEX)
                .into_iter()
                .chain(socket.receive_on_channel(
                    SilkSocketConfig::RELIABLE_CHANNEL_INDEX,
                ))
                .map(SilkSocketEvent::Message),
        );
    }
}
