use bevy::{prelude::*, time::fixed_timestep::FixedTime};
use events::{SilkBroadcastEvent, SilkServerEvent};
use signaler::SilkSignalerPlugin;
use silk_common::bevy_matchbox::{
    matchbox_socket::{PeerId, PeerState},
    prelude::MultipleChannels,
    MatchboxSocket, OpenSocketExt,
};
use silk_common::{ConnectionAddr, SilkSocket};

pub mod events;
pub mod signaler;
mod stages;
pub use stages::*;

/// The socket server abstraction
pub struct SilkServerPlugin {
    /// Whether the signaling server is local or remote
    pub signaler_addr: ConnectionAddr,
    /// Hertz for server tickrate, e.g. 30.0 = 30 times per second
    pub tick_rate: f32,
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
        if let ConnectionAddr::Local { port } = self.signaler_addr {
            app.add_plugin(SilkSignalerPlugin { port });
        }
        app.insert_resource(SocketState {
            addr: self.signaler_addr,
            id: None,
        })
        .add_startup_system(init_socket)
        .insert_resource(FixedTime::new_from_secs(1.0 / self.tick_rate))
        .add_event::<SilkServerEvent>()
        .add_event::<SilkBroadcastEvent>();

        app.init_schedule(SilkStagesSchedule);

        // it's important here to configure set order
        app.edit_schedule(SilkStagesSchedule, |schedule| {
            schedule.configure_sets(SilkStages::sets());
        });

        app.add_systems(
            (socket_reader, trace_read)
                .in_base_set(SilkStages::ReadSocket)
                .in_schedule(SilkStagesSchedule),
        )
        .add_system(
            trace_incoming
                .in_base_set(SilkStages::ProcessIncomingEvents)
                .in_schedule(SilkStagesSchedule),
        )
        .add_system(
            trace_update_state
                .in_base_set(SilkStages::UpdateWorldState)
                .in_schedule(SilkStagesSchedule),
        )
        .add_system(
            trace_outgoing
                .in_base_set(SilkStages::ProcessOutgoingEvents)
                .in_schedule(SilkStagesSchedule),
        )
        .add_systems(
            (broadcast, trace_write)
                .in_base_set(SilkStages::WriteSocket)
                .in_schedule(SilkStagesSchedule),
        );

        // add scheduler
        app.add_system(
            stages::run_silk_schedule
                .in_schedule(CoreSchedule::FixedUpdate)
                .before(bevy::time::fixed_timestep::run_fixed_update_schedule),
        );
    }
}

fn trace_read() {
    trace!("Trace 1: Read");
}

fn trace_incoming() {
    trace!("Trace 2: Incoming");
}

fn trace_update_state() {
    trace!("Trace 3: Update");
}

fn trace_outgoing() {
    trace!("Trace 4: Outgoing");
}

fn trace_write() {
    trace!("Trace 5: Write");
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
    let reliable_msgs =
        socket.channel(SilkSocket::RELIABLE_CHANNEL_INDEX).receive();
    let unreliable_msgs = socket
        .channel(SilkSocket::UNRELIABLE_CHANNEL_INDEX)
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
            // Unreliable operations
            SilkBroadcastEvent::UnreliableSendAllExcept((peer, packet)) => {
                let peers: Vec<PeerId> =
                    socket.connected_peers().filter(|p| p != peer).collect();
                peers.into_iter().for_each(|peer| {
                    socket
                        .channel(SilkSocket::UNRELIABLE_CHANNEL_INDEX)
                        .send(packet.clone(), peer)
                })
            }
            SilkBroadcastEvent::UnreliableSendAll(packet) => {
                let peers: Vec<PeerId> = socket.connected_peers().collect();
                peers.into_iter().for_each(|peer| {
                    socket
                        .channel(SilkSocket::UNRELIABLE_CHANNEL_INDEX)
                        .send(packet.clone(), peer)
                })
            }
            SilkBroadcastEvent::UnreliableSend((peer, packet)) => socket
                .channel(SilkSocket::UNRELIABLE_CHANNEL_INDEX)
                .send(packet.clone(), *peer),

            // Reliable operations
            SilkBroadcastEvent::ReliableSendAllExcept((peer, packet)) => {
                let peers: Vec<PeerId> =
                    socket.connected_peers().filter(|p| p != peer).collect();
                peers.into_iter().for_each(|peer| {
                    socket
                        .channel(SilkSocket::RELIABLE_CHANNEL_INDEX)
                        .send(packet.clone(), peer)
                })
            }
            SilkBroadcastEvent::ReliableSendAll(packet) => {
                let peers: Vec<PeerId> = socket.connected_peers().collect();
                peers.into_iter().for_each(|peer| {
                    socket
                        .channel(SilkSocket::RELIABLE_CHANNEL_INDEX)
                        .send(packet.clone(), peer)
                })
            }
            SilkBroadcastEvent::ReliableSend((peer, packet)) => socket
                .channel(SilkSocket::RELIABLE_CHANNEL_INDEX)
                .send(packet.clone(), *peer),
        }
    }
}
