use std::net::IpAddr;

use bevy::{prelude::*, tasks::IoTaskPool, time::FixedTimestep};
use events::{SilkBroadcastEvent, SilkServerEvent};
use matchbox_socket::{PeerState, WebRtcSocket};
use silk_common::{SilkSocket, SilkSocketConfig};
use state::ServerState;
pub mod events;
pub mod state;

pub struct SilkServerPlugin {
    /// Whether the signalling server is local or remote
    pub signalling_server: Option<IpAddr>,
    /// The port to serve
    pub port: u16,
    /// Hertz for server tickrate, e.g. 30.0 = 30 times per second
    pub tick_rate: f64,
}

#[derive(Resource)]
pub struct SocketResource {
    // The ID the signalling server sees us as
    pub id: Option<String>,
    // The underlying matchbox socket being translated
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
        app.add_stage_after(
            CoreStage::First,
            stages::READ_SOCKET,
            SystemStage::parallel().with_run_criteria(
                FixedTimestep::steps_per_second(self.tick_rate),
            ),
        );
        app.add_stage_after(
            CoreStage::PreUpdate,
            stages::PROCESS_INCOMING_EVENTS,
            SystemStage::parallel().with_run_criteria(
                FixedTimestep::steps_per_second(self.tick_rate),
            ),
        );
        app.add_stage_after(
            CoreStage::Update,
            stages::UPDATE_WORLD_STATE,
            SystemStage::parallel().with_run_criteria(
                FixedTimestep::steps_per_second(self.tick_rate),
            ),
        );
        app.add_stage_after(
            CoreStage::Update,
            stages::PROCESS_OUTGOING_EVENTS,
            SystemStage::parallel().with_run_criteria(
                FixedTimestep::steps_per_second(self.tick_rate),
            ),
        );
        app.add_stage_after(
            CoreStage::Update,
            stages::WRITE_SOCKET,
            SystemStage::parallel().with_run_criteria(
                FixedTimestep::steps_per_second(self.tick_rate),
            ),
        );

        let config = match self.signalling_server {
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

        app.insert_resource(ServerState::default())
            .insert_resource(SocketResource {
                id: None,
                mb_socket: socket,
            })
            .add_event::<SilkServerEvent>()
            .add_system_to_stage(stages::READ_SOCKET, receive)
            .add_event::<SilkBroadcastEvent>()
            .add_system_to_stage(stages::WRITE_SOCKET, broadcast);

        // TODO: run the "broadcast" function after the receive system
    }
}

pub fn receive(
    mut state: ResMut<ServerState>,
    mut socket_res: ResMut<SocketResource>,
    mut event_wtr: EventWriter<SilkServerEvent>,
) {
    let state = state.as_mut();
    let socket = socket_res.as_mut();

    // Check if we received an ID from signaller
    if let Some(id) = socket.mb_socket.id() {
        if socket.id.is_none() {
            socket.id.replace(id);
        }
    }

    // Check for peer updates
    for (peer, peer_state) in socket.mb_socket.update_peers() {
        match peer_state {
            PeerState::Connected => {
                info!("Peer joined: {:?}", peer);
                state.clients.insert(peer.clone());
                event_wtr.send(SilkServerEvent::PeerJoined(peer));
            }
            PeerState::Disconnected => {
                info!("Peer left: {:?}", peer);
                state.clients.remove(&peer);
                event_wtr.send(SilkServerEvent::PeerLeft(peer));
            }
        }
    }

    // Check for new messages
    for (peer, packet) in socket
        .mb_socket
        .receive_on_channel(SilkSocketConfig::RELIABLE_CHANNEL_INDEX)
    {
        info!(
            "Received from {:?}: {:?}",
            peer,
            String::from_utf8_lossy(&packet)
        );
        event_wtr.send(SilkServerEvent::MessageReceived(packet));
    }
}

pub fn broadcast(
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
