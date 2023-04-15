use bevy::{prelude::*, time::fixed_timestep::FixedTime};
use events::SilkBroadcastEvent;
use signaler::SilkSignalerPlugin;
use silk_common::{
    bevy_matchbox::{
        matchbox_socket::{PeerId, PeerState},
        prelude::MultipleChannels,
        MatchboxSocket, OpenSocketExt,
    },
    schedule::*,
    SilkCommonPlugin, SilkSocketEvent, SilkStage,
};
use silk_common::{ConnectionAddr, SilkSocket};

pub mod events;
pub mod signaler;

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
        app.add_plugin(SilkCommonPlugin);

        if let ConnectionAddr::Local { port } = self.signaler_addr {
            app.add_plugin(SilkSignalerPlugin { port });
        }
        app.insert_resource(SocketState {
            addr: self.signaler_addr,
            id: None,
        })
        .add_startup_system(init_socket)
        .insert_resource(FixedTime::new_from_secs(1.0 / self.tick_rate))
        .add_event::<SilkSocketEvent>()
        .add_event::<SilkBroadcastEvent>();

        app.add_system(
            trace_read.before(socket_reader).in_schedule(SilkSchedule),
        )
        .add_system(
            // Read silk events always before servers, who hook into this stage
            socket_reader
                .before(SilkStage::ReadIn)
                .in_schedule(SilkSchedule),
        )
        .add_system(
            trace_incoming
                .after(SilkStage::ReadIn)
                .before(SilkStage::Process)
                .in_schedule(SilkSchedule),
        )
        .add_system(
            trace_update_state
                .after(SilkStage::Process)
                .before(SilkStage::Update)
                .in_schedule(SilkSchedule),
        )
        .add_system(
            trace_write
                .after(SilkStage::Update)
                .before(SilkStage::WriteOut)
                .in_schedule(SilkSchedule),
        );
        // .add_system(
        //     // Write silk events always after servers, who hook into this stage
        //     broadcast
        //         .after(SilkStage::WriteOut)
        //         .in_schedule(SilkSchedule),
        // );
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
    mut event_wtr: EventWriter<SilkSocketEvent>,
) {
    // Id changed events
    if let Some(id) = socket.id() {
        if state.id.is_none() {
            state.id.replace(id);
            event_wtr.send(SilkSocketEvent::IdAssigned(id));
        }
    }

    // Check for peer updates
    for (peer, peer_state) in socket.update_peers() {
        match peer_state {
            PeerState::Connected => {
                event_wtr.send(SilkSocketEvent::ClientJoined(peer));
            }
            PeerState::Disconnected => {
                event_wtr.send(SilkSocketEvent::ClientLeft(peer));
            }
        }
    }
}

// /// Reads and handles server broadcast request events
// fn broadcast(
//     mut socket: ResMut<MatchboxSocket<MultipleChannels>>,
//     mut event_reader: EventReader<SilkBroadcastEvent>,
// ) {
//     trace!("Trace 5: Broadcasting {} events", event_reader.len());
//     while let Some(broadcast) = event_reader.iter().next() {
//         match broadcast {
//             // Unreliable operations
//             SilkBroadcastEvent::UnreliableSendAllExcept((peer, packet)) => {
//                 let peers: Vec<PeerId> =
//                     socket.connected_peers().filter(|p| p != peer).collect();
//                 peers.into_iter().for_each(|peer| {
//                     socket
//                         .channel(SilkSocket::UNRELIABLE_CHANNEL_INDEX)
//                         .send(packet.clone(), peer)
//                 })
//             }
//             SilkBroadcastEvent::UnreliableSendAll(packet) => {
//                 let peers: Vec<PeerId> = socket.connected_peers().collect();
//                 peers.into_iter().for_each(|peer| {
//                     socket
//                         .channel(SilkSocket::UNRELIABLE_CHANNEL_INDEX)
//                         .send(packet.clone(), peer)
//                 })
//             }
//             SilkBroadcastEvent::UnreliableSend((peer, packet)) => socket
//                 .channel(SilkSocket::UNRELIABLE_CHANNEL_INDEX)
//                 .send(packet.clone(), *peer),

//             // Reliable operations
//             SilkBroadcastEvent::ReliableSendAllExcept((peer, packet)) => {
//                 let peers: Vec<PeerId> =
//                     socket.connected_peers().filter(|p| p != peer).collect();
//                 peers.into_iter().for_each(|peer| {
//                     socket
//                         .channel(SilkSocket::RELIABLE_CHANNEL_INDEX)
//                         .send(packet.clone(), peer)
//                 })
//             }
//             SilkBroadcastEvent::ReliableSendAll(packet) => {
//                 let peers: Vec<PeerId> = socket.connected_peers().collect();
//                 peers.into_iter().for_each(|peer| {
//                     socket
//                         .channel(SilkSocket::RELIABLE_CHANNEL_INDEX)
//                         .send(packet.clone(), peer)
//                 })
//             }
//             SilkBroadcastEvent::ReliableSend((peer, packet)) => socket
//                 .channel(SilkSocket::RELIABLE_CHANNEL_INDEX)
//                 .send(packet.clone(), *peer),
//         }
//     }
// }
