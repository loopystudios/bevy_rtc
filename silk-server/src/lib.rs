use bevy::{prelude::*, time::FixedTimestep};
use bevy_matchbox::{
    matchbox_socket::{PeerId, PeerState},
    prelude::*,
};
use events::{SilkBroadcastEvent, SilkServerEvent};
use silk_common::{ConnectionAddr, SilkSocket};

pub mod events;

/// The socket server abstraction
pub struct SilkServerPlugin {
    /// Whether the signaling server is local or remote
    pub signaler_addr: ConnectionAddr,
    /// Hertz for server tickrate, e.g. 30.0 = 30 times per second
    pub tick_rate: f64,
}

#[derive(Resource)]
struct SocketState {
    /// The socket address, used for connecting/reconnecting
    pub addr: ConnectionAddr,

    /// The ID the signaling server sees us as
    pub id: Option<PeerId>,
}

impl Plugin for SilkServerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SocketState {
            addr: self.signaler_addr,
            id: None,
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
        .add_system_to_stage(stages::WRITE_SOCKET, broadcast)
        .add_startup_system(init_socket);
    }
}

/// Initialize the socket
fn init_socket(mut commands: Commands, state: Res<SocketState>) {
    debug!("address: {:?}", state.addr);

    // Create matchbox socket
    let silk_socket = SilkSocket::new(state.addr);
    commands.open_socket(silk_socket.builder());
}

/// Translates socket events into Bevy events
fn socket_reader(
    mut state: ResMut<SocketState>,
    mut socket: ResMut<MatchboxSocket<MultipleChannels>>,
    mut event_wtr: EventWriter<SilkServerEvent>,
) {
    // Id changed events
    if let Some(id) = socket.id() {
        if state.id.is_none() {
            state.id.replace(id);
            event_wtr.send(SilkServerEvent::IdAssigned(id));
        }
    }

    // Check for peer updates
    for (peer, peer_state) in socket.update_peers() {
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
    let reliable_msgs = socket
        .channel(SilkSocket::RELIABLE_CHANNEL_INDEX)
        .unwrap()
        .receive();
    let unreliable_msgs = socket
        .channel(SilkSocket::UNRELIABLE_CHANNEL_INDEX)
        .unwrap()
        .receive();
    event_wtr.send_batch(
        reliable_msgs
            .into_iter()
            .chain(unreliable_msgs)
            .map(SilkServerEvent::Message),
    );
}

/// Reads and handles server broadcast request events
fn broadcast(
    mut socket: ResMut<MatchboxSocket<MultipleChannels>>,
    mut event_reader: EventReader<SilkBroadcastEvent>,
) {
    while let Some(broadcast) = event_reader.iter().next() {
        match broadcast {
            SilkBroadcastEvent::UnreliableSendAll(packet) => {
                let peers: Vec<PeerId> = socket.connected_peers().collect();
                peers.into_iter().for_each(|peer| {
                    socket
                        .channel(SilkSocket::UNRELIABLE_CHANNEL_INDEX)
                        .unwrap()
                        .send(packet.clone(), peer)
                })
            }
            SilkBroadcastEvent::UnreliableSend((peer, packet)) => socket
                .channel(SilkSocket::UNRELIABLE_CHANNEL_INDEX)
                .unwrap()
                .send(packet.clone(), *peer),
            SilkBroadcastEvent::ReliableSendAll(packet) => {
                let peers: Vec<PeerId> = socket.connected_peers().collect();
                peers.into_iter().for_each(|peer| {
                    socket
                        .channel(SilkSocket::RELIABLE_CHANNEL_INDEX)
                        .unwrap()
                        .send(packet.clone(), peer)
                })
            }
            SilkBroadcastEvent::ReliableSend((peer, packet)) => socket
                .channel(SilkSocket::RELIABLE_CHANNEL_INDEX)
                .unwrap()
                .send(packet.clone(), *peer),
        }
    }
}

pub mod stages {
    /// Silk plugin reads from silk socket and sends "incoming client
    /// message" events
    pub static READ_SOCKET: &str = "silk_read_socket";
    /// Game receives "incoming client message" events from Silk plugin
    /// and creates "side effects"
    pub static PROCESS_INCOMING_EVENTS: &str = "silk_process_incoming_events";
    /// Game updates world state here with the "side effects"
    pub static UPDATE_WORLD_STATE: &str = "silk_world_update";
    /// Game sends broadcast events to Silk plugin (after world state
    /// reacts with "side effects" to create a new world state)
    pub static PROCESS_OUTGOING_EVENTS: &str = "silk_process_outgoing_events";
    /// Silk plugin reads broadcast events game and sends messages over
    /// the silk socket
    pub static WRITE_SOCKET: &str = "silk_write_socket";
}
