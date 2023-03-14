use bevy::{prelude::*, tasks::IoTaskPool, time::FixedTimestep};
use events::{SilkBroadcastEvent, SilkServerEvent};
use matchbox_socket::{PeerState, WebRtcSocket};
use silk_common::{SilkSocket, SilkSocketConfig};
use std::net::IpAddr;
pub mod events;

/// The socket server abstraction
pub struct SilkServerPlugin {
    /// Whether the signalling server is local or remote
    pub remote_signalling_server: Option<IpAddr>,
    /// The port to serve
    pub port: u16,
    /// Hertz for server tickrate, e.g. 30.0 = 30 times per second
    pub tick_rate: f64,
}

#[derive(Resource)]
struct SocketResource {
    /// The ID the signalling server sees us as
    pub id: Option<String>,
    /// The underlying matchbox socket being translated
    pub mb_socket: WebRtcSocket,
}

pub mod stages {
    /// Silk Server plugin reads from silk socket and sends "incoming client
    /// message" events
    pub(crate) static READ_SOCKET: &str = "silk_read_socket";
    /// Game receives "incoming client message" events from Silk Server plugin
    /// and creates "side effects"
    pub static PROCESS_INCOMING_EVENTS: &str = "silk_process_incoming_events";
    /// Game updates world state here with the "side effects"
    pub static UPDATE_WORLD_STATE: &str = "silk_world_update";
    /// Game sends broadcast events to Silk Server plugin (after world state
    /// reacts with "side effects" to create a new world state)
    pub static PROCESS_OUTGOING_EVENTS: &str = "silk_process_outgoing_events";
    /// Silk Server Plugin reads broadcast events game and sends messages over
    /// the silk socket
    pub(crate) static WRITE_SOCKET: &str = "silk_write_socket";
}

impl Plugin for SilkServerPlugin {
    fn build(&self, app: &mut App) {
        let config = match self.remote_signalling_server {
            Some(ip) => SilkSocketConfig::RemoteSignallerAsHost {
                ip,
                port: self.port,
            },
            None => SilkSocketConfig::LocalSignallerAsHost { port: self.port },
        };
        let socket = SilkSocket::new(config);
        let (socket, loop_fut) = socket.into_parts();

        // The loop_fut runs the socket, and is async, so we use Bevy's polling.
        let task_pool = IoTaskPool::get();
        task_pool.spawn(loop_fut).detach();

        app.insert_resource(SocketResource {
            id: None,
            mb_socket: socket,
        })
        .add_stage_after(
            CoreStage::First,
            stages::READ_SOCKET,
            SystemStage::parallel().with_run_criteria(
                FixedTimestep::steps_per_second(self.tick_rate),
            ),
        )
        .add_stage_after(
            CoreStage::PreUpdate,
            stages::PROCESS_INCOMING_EVENTS,
            SystemStage::parallel().with_run_criteria(
                FixedTimestep::steps_per_second(self.tick_rate),
            ),
        )
        .add_stage_after(
            CoreStage::Update,
            stages::UPDATE_WORLD_STATE,
            SystemStage::parallel().with_run_criteria(
                FixedTimestep::steps_per_second(self.tick_rate),
            ),
        )
        .add_stage_after(
            CoreStage::Update,
            stages::PROCESS_OUTGOING_EVENTS,
            SystemStage::parallel().with_run_criteria(
                FixedTimestep::steps_per_second(self.tick_rate),
            ),
        )
        .add_stage_after(
            CoreStage::Update,
            stages::WRITE_SOCKET,
            SystemStage::parallel().with_run_criteria(
                FixedTimestep::steps_per_second(self.tick_rate),
            ),
        )
        .add_event::<SilkServerEvent>()
        .add_system_to_stage(stages::READ_SOCKET, socket_reader)
        .add_event::<SilkBroadcastEvent>()
        .add_system_to_stage(stages::WRITE_SOCKET, broadcast);
    }
}

/// Translates socket events into Bevy events
fn socket_reader(
    mut socket_res: ResMut<SocketResource>,
    mut event_wtr: EventWriter<SilkServerEvent>,
) {
    let socket = socket_res.as_mut();

    // Id changed events
    if let Some(id) = socket.mb_socket.id() {
        if socket.id.is_none() {
            socket.id.replace(id.clone());
            event_wtr.send(SilkServerEvent::IdAssigned(id.clone()));
        }
    }

    // Check for peer updates
    for (peer, peer_state) in socket.mb_socket.update_peers() {
        match peer_state {
            PeerState::Connected => {
                event_wtr.send(SilkServerEvent::PeerJoined(peer));
            }
            PeerState::Disconnected => {
                event_wtr.send(SilkServerEvent::PeerLeft(peer));
            }
        }
    }

    // Collect Unreliable, Reliable messages
    event_wtr.send_batch(
        socket
            .mb_socket
            .receive_on_channel(SilkSocketConfig::UNRELIABLE_CHANNEL_INDEX)
            .into_iter()
            .chain(
                socket.mb_socket.receive_on_channel(
                    SilkSocketConfig::RELIABLE_CHANNEL_INDEX,
                ),
            )
            .map(SilkServerEvent::Message),
    );
}

/// Reads and handles server broadcast request events
fn broadcast(
    mut socket_res: ResMut<SocketResource>,
    mut event_reader: EventReader<SilkBroadcastEvent>,
) {
    let socket = socket_res.as_mut();

    while let Some(broadcast) = event_reader.iter().next() {
        match broadcast {
            SilkBroadcastEvent::UnreliableSendAll(packet) => {
                socket.mb_socket.broadcast_on_channel(
                    packet.clone(),
                    SilkSocketConfig::UNRELIABLE_CHANNEL_INDEX,
                )
            }
            SilkBroadcastEvent::UnreliableSend((peer, packet)) => {
                socket.mb_socket.send_on_channel(
                    packet.clone(),
                    peer,
                    SilkSocketConfig::UNRELIABLE_CHANNEL_INDEX,
                )
            }
            SilkBroadcastEvent::ReliableSendAll(packet) => {
                socket.mb_socket.broadcast_on_channel(
                    packet.clone(),
                    SilkSocketConfig::RELIABLE_CHANNEL_INDEX,
                )
            }
            SilkBroadcastEvent::ReliableSend((peer, packet)) => {
                socket.mb_socket.send_on_channel(
                    packet.clone(),
                    peer,
                    SilkSocketConfig::RELIABLE_CHANNEL_INDEX,
                )
            }
        }
    }
}
