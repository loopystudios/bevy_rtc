use bevy::{log::LogPlugin, prelude::*, time::FixedTimestep, utils::HashSet};
use matchbox_socket::PeerId;
use silk_server::{
    events::{SilkBroadcastEvent, SilkServerEvent},
    stages, SilkServerPlugin,
};

#[derive(Resource, Debug, Default, Clone)]
struct ServerState {
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
            stages::PROCESS_INCOMING_EVENTS,
            handle_events,
        )
        //.add_system(
        //    broadcast_to_peers
        //        .with_run_criteria(FixedTimestep::steps_per_second(0.2)), // Every 5s
        //)
        .insert_resource(ServerState::default())
        .add_startup_system(|| info!("Connecting..."))
        .run();
}

fn broadcast_to_peers(
    mut event_wtr: EventWriter<SilkBroadcastEvent>,
    world_state: Res<ServerState>,
) {
    for client in world_state.clients.iter() {
        let packet = format!(
            "Hello {client:?}, the server has {} clients",
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
    mut world_state: ResMut<ServerState>,
) {
    while let Some(ev) = event_rdr.iter().next() {
        match ev {
            SilkServerEvent::PeerJoined(id) => {
                world_state.clients.insert(*id);
                debug!("{id:?} joined");
                let packet =
                    "someone joined!".as_bytes().to_vec().into_boxed_slice();
                event_wtr.send(SilkBroadcastEvent::ReliableSendAll(packet));
            }
            SilkServerEvent::PeerLeft(id) => {
                debug!("{id:?} left");
                world_state.clients.remove(id);
                let packet =
                    "someone left!".as_bytes().to_vec().into_boxed_slice();
                event_wtr.send(SilkBroadcastEvent::ReliableSendAll(packet));
            }
            SilkServerEvent::Message((id, packet)) => {
                let msg = String::from_utf8_lossy(packet); // last char is /n
                debug!("{id:?}: {msg}");
                let packet = msg.as_bytes().to_vec().into_boxed_slice();
                for peer in world_state.clients.iter().filter(|p| p != &id) {
                    event_wtr.send(SilkBroadcastEvent::ReliableSend((
                        *peer,
                        packet.clone(),
                    )));
                }
            }
            SilkServerEvent::IdAssigned(id) => info!("I am {id:?}"),
        }
    }
    event_rdr.clear();
}
