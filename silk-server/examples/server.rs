use bevy::{log::LogPlugin, prelude::*, time::FixedTimestep, utils::HashSet};
use matchbox_socket::PeerId;
use silk_server::{
    events::{SilkBroadcastEvent, SilkServerEvent},
    SilkServerPlugin,
};

#[derive(Resource, Debug, Default, Clone)]
struct WorldState {
    clients: HashSet<PeerId>,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugin(LogPlugin {
            filter: "error,server=debug,wgpu_core=warn,wgpu_hal=warn,matchbox_socket=warn"
                .into(),
            level: bevy::log::Level::DEBUG,
        })
        .add_plugin(SilkServerPlugin {
            port: 3536,
            tick_rate: 5.0,
            remote_signalling_server: None,
        })
        .add_system_to_stage(
            silk_server::stages::PROCESS_INCOMING_EVENTS,
            handle_events,
        )
        .add_system(
            broadcast_to_peers
                .with_run_criteria(FixedTimestep::steps_per_second(0.2)), // Every 5s
        )
        .insert_resource(WorldState::default())
        .add_startup_system(|| info!("Connecting..."))
        .run();
}

fn broadcast_to_peers(
    mut event_wtr: EventWriter<SilkBroadcastEvent>,
    world_state: Res<WorldState>,
) {
    for client in world_state.clients.iter() {
        let packet = format!(
            "Hello {}, the server has {} clients",
            client,
            world_state.clients.len()
        )
        .as_bytes()
        .to_vec()
        .into_boxed_slice();
        event_wtr.send(SilkBroadcastEvent::ReliableSendAll(packet));
    }
}

fn handle_events(
    mut event_rdr: EventReader<SilkServerEvent>,
    mut event_wtr: EventWriter<SilkBroadcastEvent>,
    mut world_state: ResMut<WorldState>,
) {
    while let Some(ev) = event_rdr.iter().next() {
        match ev {
            SilkServerEvent::PeerJoined(id) => {
                world_state.clients.insert(id.clone());
                debug!("{id} joined");
                let packet =
                    "someone joined!".as_bytes().to_vec().into_boxed_slice();
                event_wtr.send(SilkBroadcastEvent::ReliableSendAll(packet));
            }
            SilkServerEvent::PeerLeft(id) => {
                debug!("{id} left");
                world_state.clients.remove(id);
                let packet =
                    "someone left!".as_bytes().to_vec().into_boxed_slice();
                event_wtr.send(SilkBroadcastEvent::ReliableSendAll(packet));
            }
            SilkServerEvent::Message((id, packet)) => {
                let msg = String::from_utf8_lossy(&packet[0..packet.len() - 1]); // last char is /n
                debug!("{id}: {msg}");
                let packet =
                    "message received!".as_bytes().to_vec().into_boxed_slice();
                event_wtr.send(SilkBroadcastEvent::ReliableSendAll(packet));
            }
            SilkServerEvent::IdAssigned(id) => info!("I am {id}"),
        }
    }
    event_rdr.clear();
}
