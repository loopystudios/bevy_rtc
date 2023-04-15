use bevy::prelude::*;
use events::{SilkSendEvent, SilkSocketEvent};
use schedule::{SilkClientSchedule, SilkClientStage};
use silk_common::bevy_matchbox::{matchbox_socket, prelude::*};
use silk_common::packets::SilkPayload;
use silk_common::{ConnectionAddr, PlayerAuthentication, SilkSocket};
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
    Establishing,
    LoggingIn,
    Connected,
}

impl Plugin for SilkClientPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SocketState::default())
            .add_state::<ConnectionState>()
            .add_event::<ConnectionRequest>()
            .add_event::<SilkSocketEvent>()
            .add_event::<SilkSendEvent>()
            .add_system(connection_event_reader)
            .add_system(
                init_socket.in_schedule(OnEnter(ConnectionState::Establishing)),
            )
            .add_system(
                reset_socket
                    .in_schedule(OnEnter(ConnectionState::Disconnected)),
            );

        app.init_schedule(SilkClientSchedule);

        // it's important here to configure set order
        app.edit_schedule(SilkClientSchedule, |schedule| {
            schedule.configure_sets(SilkClientStage::sets());
        });

        app.add_system(
            trace_read
                .before(socket_reader)
                .in_schedule(SilkClientSchedule),
        )
        .add_system(
            // Read silk events always before clients, who hook into this stage
            socket_reader
                .before(SilkClientStage::ReadIn)
                .in_schedule(SilkClientSchedule),
        )
        .add_system(
            trace_incoming
                .after(SilkClientStage::ReadIn)
                .before(SilkClientStage::ProcessLatency)
                .in_schedule(SilkClientSchedule),
        )
        .add_system(
            trace_update_state
                .after(SilkClientStage::ProcessLatency)
                .before(SilkClientStage::Update)
                .in_schedule(SilkClientSchedule),
        )
        .add_system(
            trace_write
                .after(SilkClientStage::Update)
                .before(SilkClientStage::WriteOut)
                .in_schedule(SilkClientSchedule),
        )
        .add_system(
            // Write silk events always after clients, who hook into this stage
            socket_writer
                .after(SilkClientStage::WriteOut)
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

fn trace_incoming() {
    trace!("Trace 2: Latency Processing");
}

fn trace_update_state() {
    trace!("Trace 3: Update");
}

fn trace_write() {
    trace!("Trace 4: Write");
}

#[derive(Resource, Default)]
struct SocketState {
    /// The socket address, used for connecting/reconnecting
    pub addr: Option<ConnectionAddr>,
    /// The authentication provided to connect
    pub auth: Option<PlayerAuthentication>,
    /// The ID of the host
    pub host_id: Option<PeerId>,
    /// The ID given by the signaling server
    pub id: Option<PeerId>,
}

pub enum ConnectionRequest {
    /// A request to connect to the server through the signaling server; the
    /// ip and port are the signaling server
    Connect {
        ip: IpAddr,
        port: u16,
        auth: PlayerAuthentication,
    },
    /// A request to disconnect from the signaling server; this will also
    /// disconnect from the server
    Disconnect { reason: Option<String> },
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
    *state = SocketState::default();
}

/// Reads and handles connection request events
fn connection_event_reader(
    mut cxn_event_reader: EventReader<ConnectionRequest>,
    mut state: ResMut<SocketState>,
    mut next_connection_state: ResMut<NextState<ConnectionState>>,
    current_connection_state: Res<State<ConnectionState>>,
    mut silk_event_wtr: EventWriter<SilkSocketEvent>,
) {
    match cxn_event_reader.iter().next() {
        Some(ConnectionRequest::Connect { ip, port, auth }) => {
            if let ConnectionState::Disconnected = current_connection_state.0 {
                let addr = ConnectionAddr::Remote {
                    ip: *ip,
                    port: *port,
                };
                debug!(
                    previous = format!("{current_connection_state:?}"),
                    "set state: connecting"
                );
                state.addr.replace(addr);
                state.auth.replace(auth.clone());
                next_connection_state.set(ConnectionState::Establishing);
            }
        }
        Some(ConnectionRequest::Disconnect { reason }) => {
            debug!(
                previous = format!("{current_connection_state:?}"),
                "set state: disconnected"
            );
            silk_event_wtr.send(SilkSocketEvent::DisconnectedFromHost {
                reason: reason.clone(),
            });
            next_connection_state.set(ConnectionState::Disconnected);
        }
        None => {}
    }
}

/// Translates socket updates into bevy events
fn socket_reader(
    mut state: ResMut<SocketState>,
    mut socket: Option<ResMut<MatchboxSocket<MultipleChannels>>>,
    mut event_wtr: EventWriter<SilkSocketEvent>,
    mut send_wtr: EventWriter<SilkSendEvent>,
    connection_state: Res<State<ConnectionState>>,
    mut next_connection_state: ResMut<NextState<ConnectionState>>,
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
                    if state.host_id.is_some() {
                        panic!("server already connected");
                    }
                    state.host_id.replace(id);
                    next_connection_state.set(ConnectionState::LoggingIn);
                    match state.auth.as_ref().expect("auth not set") {
                        PlayerAuthentication::Registered {
                            username,
                            password,
                            mfa,
                        } => send_wtr.send(SilkSendEvent::ReliableSend(
                            SilkPayload::AuthenticateUser {
                                username: username.clone(),
                                password: password.clone(),
                                mfa: mfa.clone(),
                            },
                        )),
                        PlayerAuthentication::Guest { username } => send_wtr
                            .send(SilkSendEvent::ReliableSend(
                                SilkPayload::AuthenticateGuest {
                                    username: username.clone(),
                                },
                            )),
                    }
                }
                matchbox_socket::PeerState::Disconnected => {
                    if state.host_id.is_none() {
                        panic!("server wasn't connected!");
                    }
                    state.host_id.take();
                    next_connection_state.set(ConnectionState::Disconnected);
                    event_wtr.send(SilkSocketEvent::DisconnectedFromHost {
                        reason: Some("Server reset".to_string()),
                    });
                }
            }
        }

        // Collect Unreliable, Reliable messages
        let messages = socket
            .channel(SilkSocket::RELIABLE_CHANNEL_INDEX)
            .receive()
            .into_iter()
            .chain(
                socket
                    .channel(SilkSocket::UNRELIABLE_CHANNEL_INDEX)
                    .receive(),
            );

        // Route requests
        for (peer, packet) in messages {
            let Ok(payload) = SilkPayload::try_from(&packet) else {
                error!("bad packet: {packet:?}");
                continue;
            };
            match connection_state.0 {
                // Respond to anything
                ConnectionState::Connected => {
                    let SilkPayload::Message(packet) = payload else {
                        error!("in connected state and received non-message: {packet:?}");
                        continue;
                    };
                    event_wtr.send(SilkSocketEvent::Message((peer, packet)))
                }
                // Only respond to authentication
                ConnectionState::LoggingIn => match payload {
                    SilkPayload::LoginAccepted { username } => {
                        next_connection_state.set(ConnectionState::Connected);
                        event_wtr.send(SilkSocketEvent::ConnectedToHost {
                            host: state.host_id.unwrap(),
                            username,
                        });
                    }
                    SilkPayload::LoginDenied { reason } => {
                        next_connection_state
                            .set(ConnectionState::Disconnected);
                        event_wtr.send(SilkSocketEvent::DisconnectedFromHost {
                            reason: Some(reason),
                        });
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}

/// Sends aggregated messages requested by the client
fn socket_writer(
    mut socket: Option<ResMut<MatchboxSocket<MultipleChannels>>>,
    state: Res<SocketState>,
    current_connection_state: Res<State<ConnectionState>>,
    mut silk_event_rdr: EventReader<SilkSendEvent>,
) {
    let Some(socket) = socket.as_mut() else { return };
    let (ConnectionState::LoggingIn | ConnectionState::Connected) = current_connection_state.0 else { return };
    let Some(host) = state.host_id else { return };
    trace!("Trace 3: Sending {} messages", silk_event_rdr.len());
    for ev in silk_event_rdr.iter() {
        match ev {
            SilkSendEvent::ReliableSend(data) => {
                socket
                    .channel(SilkSocket::RELIABLE_CHANNEL_INDEX)
                    .send(data.into_packet(), host);
            }
            SilkSendEvent::UnreliableSend(data) => {
                socket
                    .channel(SilkSocket::UNRELIABLE_CHANNEL_INDEX)
                    .send(data.into_packet(), host);
            }
        }
    }
}
