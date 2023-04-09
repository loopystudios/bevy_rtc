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
pub mod schedule;
pub mod signaler;
pub use schedule::*;

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

        app.init_schedule(SilkServerSchedule);

        // it's important here to configure set order
        app.edit_schedule(SilkServerSchedule, |schedule| {
            schedule.configure_sets(SilkServerStage::sets());
        });

        app.add_systems(
            (socket_reader, trace_read)
                .before(SilkServerStage::ReadSocket)
                .in_schedule(SilkServerSchedule),
        )
        .add_system(
            trace_incoming
                .after(SilkServerStage::ReadSocket)
                .before(SilkServerStage::ProcessIncomingEvents)
                .in_schedule(SilkServerSchedule),
        )
        .add_system(
            trace_update_state
                .after(SilkServerStage::ProcessIncomingEvents)
                .before(SilkServerStage::UpdateWorldState)
                .in_schedule(SilkServerSchedule),
        )
        .add_system(
            trace_outgoing
                .after(SilkServerStage::UpdateWorldState)
                .before(SilkServerStage::ProcessOutgoingEvents)
                .in_schedule(SilkServerSchedule),
        )
        .add_systems(
            (broadcast, trace_write)
                .after(SilkServerStage::WriteSocket)
                .in_schedule(SilkServerSchedule),
        );

        // add scheduler
        app.add_system(
            schedule::run_silk_schedule.in_schedule(CoreSchedule::FixedUpdate),
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
                event_wtr.send(SilkServerEvent::ClientJoined(peer));
            }
            PeerState::Disconnected => {
                event_wtr.send(SilkServerEvent::ClientLeft(peer));
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
