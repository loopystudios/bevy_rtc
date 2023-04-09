use bevy::prelude::*;
use events::{SilkSendEvent, SilkSocketEvent};
use schedule::{SilkClientSchedule, SilkClientStage};
use silk_common::bevy_matchbox::{matchbox_socket, prelude::*};
use silk_common::{ConnectionAddr, SilkSocket};
use std::net::IpAddr;

pub mod events;
pub mod schedule;

/// The socket client abstraction
pub struct SilkClientPlugin;

/// State of the socket
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash, States)]
enum ConnectionState {
    #[default]
    Disconnected,
    Connecting,
    Connected,
}

impl Plugin for SilkClientPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SocketState::default())
            .add_state::<ConnectionState>()
            .add_event::<ConnectionRequest>()
            .add_event::<SilkSocketEvent>()
            .add_event::<SilkSendEvent>()
            .add_system(event_sender)
            .add_system(
                init_socket.in_schedule(OnEnter(ConnectionState::Connecting)),
            )
            .add_system(
                reset_socket
                    .in_schedule(OnEnter(ConnectionState::Disconnected)),
            );

        app.add_systems(
            (event_reader, trace_read)
                .in_base_set(SilkClientStage::ReadSocket)
                .in_schedule(SilkClientSchedule),
        )
        .add_system(
            trace_update_state
                .in_base_set(SilkClientStage::UpdateWorldState)
                .in_schedule(SilkClientSchedule),
        )
        .add_systems(
            (event_writer, trace_write)
                .in_base_set(SilkClientStage::WriteSocket)
                .in_schedule(SilkClientSchedule),
        );

        // add scheduler
        app.add_system(
            schedule::run_silk_schedule.in_schedule(CoreSchedule::Main),
        );
    }
}

fn trace_read() {
    trace!("Trace 1: Read");
}

fn trace_update_state() {
    trace!("Trace 2: Update");
}

fn trace_write() {
    trace!("Trace 3: Write");
}

#[derive(Resource, Default)]
struct SocketState {
    /// The socket address, used for connecting/reconnecting
    pub addr: Option<ConnectionAddr>,
    /// The ID of the host
    pub host_id: Option<PeerId>,
    /// The ID given by the signaling server
    pub id: Option<PeerId>,
}

pub enum ConnectionRequest {
    /// A request to connect to the server through the signaling server; the
    /// ip and port are the signaling server
    Connect { ip: IpAddr, port: u16 },
    /// A request to disconnect from the signaling server; this will also
    /// disconnect from the server
    Disconnect,
}

/// Initialize the socket
fn init_socket(mut commands: Commands, socket_res: Res<SocketState>) {
    if let Some(addr) = &socket_res.addr {
        debug!("address: {addr:?}");

        // Create matchbox socket
        let silk_socket = SilkSocket::new(*addr);
        commands.open_socket(silk_socket.builder());
    } else {
        panic!("state set to connecting without config");
    }
}

/// Reset the internal socket
fn reset_socket(mut commands: Commands, mut state: ResMut<SocketState>) {
    commands.close_socket::<MultipleChannels>();
    *state = SocketState {
        host_id: None,
        id: None,
        addr: state.addr.take(),
    };
}

/// Reads and handles connection request events
fn event_sender(
    mut socket: Option<ResMut<MatchboxSocket<MultipleChannels>>>,
    state: Res<SocketState>,
    current_connection_state: Res<State<ConnectionState>>,
    mut silk_event_rdr: EventReader<SilkSendEvent>,
) {
    let Some(socket) = socket.as_mut() else { return };
    let ConnectionState::Connected = current_connection_state.0 else { return };
    let Some(host) = state.host_id else { return };
    match silk_event_rdr.iter().next() {
        Some(SilkSendEvent::ReliableSend(data)) => {
            socket
                .channel(SilkSocket::RELIABLE_CHANNEL_INDEX)
                .send(data.clone(), host);
        }
        Some(SilkSendEvent::UnreliableSend(data)) => {
            socket
                .channel(SilkSocket::UNRELIABLE_CHANNEL_INDEX)
                .send(data.clone(), host);
        }
        None => {}
    }
}

/// Reads and handles connection request events
fn event_reader(
    mut cxn_event_reader: EventReader<ConnectionRequest>,
    mut state: ResMut<SocketState>,
    mut next_connection_state: ResMut<NextState<ConnectionState>>,
    current_connection_state: Res<State<ConnectionState>>,
    mut silk_event_wtr: EventWriter<SilkSocketEvent>,
) {
    match cxn_event_reader.iter().next() {
        Some(ConnectionRequest::Connect { ip, port }) => {
            if let ConnectionState::Disconnected = current_connection_state.0 {
                let addr = ConnectionAddr::Remote {
                    ip: *ip,
                    port: *port,
                };
                debug!(
                    previous = format!("{current_connection_state:?}"),
                    "set state: connecting"
                );
                state.addr = Some(addr);
                next_connection_state.set(ConnectionState::Connecting);
            }
        }
        Some(ConnectionRequest::Disconnect) => {
            if let ConnectionState::Connected = current_connection_state.0 {
                debug!(
                    previous = format!("{current_connection_state:?}"),
                    "set state: disconnected"
                );
                silk_event_wtr.send(SilkSocketEvent::DisconnectedFromHost);
                next_connection_state.set(ConnectionState::Disconnected);
            }
        }
        None => {}
    }
}

/// Translates socket updates into bevy events
fn event_writer(
    mut state: ResMut<SocketState>,
    mut socket: Option<ResMut<MatchboxSocket<MultipleChannels>>>,
    mut event_wtr: EventWriter<SilkSocketEvent>,
    mut connection_state: ResMut<NextState<ConnectionState>>,
) {
    // Create socket events for Silk
    if let Some(socket) = socket.as_mut() {
        // Id changed events
        if let Some(id) = socket.id() {
            if state.id.is_none() {
                state.id.replace(id);
                event_wtr.send(SilkSocketEvent::IdAssigned(id));
            }
        }

        // Connection state updates
        for (id, peer_state) in socket.update_peers() {
            match peer_state {
                matchbox_socket::PeerState::Connected => {
                    state.host_id.replace(id);
                    connection_state.set(ConnectionState::Connected);
                    event_wtr.send(SilkSocketEvent::ConnectedToHost(id));
                }
                matchbox_socket::PeerState::Disconnected => {
                    state.host_id.take();
                    connection_state.set(ConnectionState::Disconnected);
                    event_wtr.send(SilkSocketEvent::DisconnectedFromHost);
                }
            }
        }

        // Collect Unreliable, Reliable messages
        let reliable_msgs =
            socket.channel(SilkSocket::RELIABLE_CHANNEL_INDEX).receive();
        let unreliable_msgs = socket
            .channel(SilkSocket::UNRELIABLE_CHANNEL_INDEX)
            .receive();
        event_wtr.send_batch(
            reliable_msgs
                .into_iter()
                .chain(unreliable_msgs)
                .map(SilkSocketEvent::Message),
        );
    }
}
