use bevy::{log::LogPlugin, prelude::*, utils::HashSet};
use silk_common::packets::SilkPayload;
use silk_common::{
    bevy_matchbox::prelude::PeerId, demo_packets::PaintingDemoPayload,
    ConnectionAddr,
};
use silk_server::schedule::{SilkServerSchedule, SilkServerStage};
use silk_server::{
    events::{SilkBroadcastEvent, SilkServerEvent},
    SilkServerPlugin,
};

#[derive(Resource, Debug, Default, Clone)]
struct ServerState {
    server_id: Option<PeerId>,
    clients: HashSet<PeerId>,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugin(LogPlugin {
            filter: "warn,silk=trace,painting_server=debug,wgpu_core=warn,wgpu_hal=warn,matchbox_socket=warn"
                .into(),
            level: bevy::log::Level::DEBUG,
        })
        .add_plugin(SilkServerPlugin {
            signaler_addr: ConnectionAddr::Local { port: 3536 },
            tick_rate: 1.0,
        })
        .add_system(
            handle_events
                .in_base_set(SilkServerStage::WriteOut)
                .in_schedule(SilkServerSchedule),
        )
        .insert_resource(ServerState::default())
        .add_startup_system(|| info!("Connecting..."))
        .run();
}

fn handle_events(
    mut guest_counter: Local<u64>,
    mut event_rdr: EventReader<SilkServerEvent>,
    mut event_wtr: EventWriter<SilkBroadcastEvent>,
    mut world_state: ResMut<ServerState>,
) {
    while let Some(ev) = event_rdr.iter().next() {
        match ev {
            SilkServerEvent::GuestLoginRequest { peer_id, .. }
            | SilkServerEvent::LoginRequest { peer_id, .. } => {
                *guest_counter += 1;
                let username = format!("User-{}", *guest_counter);
                debug!(peer = format!("{peer_id:?}"), "{username:?} joined");
                let acceptance_packet = SilkPayload::LoginAccepted {
                    username: username.clone(),
                };
                event_wtr.send(SilkBroadcastEvent::ReliableSend((
                    *peer_id,
                    acceptance_packet,
                )));
                world_state.clients.insert(*peer_id);
                let packet = PaintingDemoPayload::Chat {
                    from: format!("{:?}", world_state.server_id.unwrap()),
                    message: format!("Hello {username}!"),
                };
                event_wtr.send(SilkBroadcastEvent::ReliableSendAll(
                    SilkPayload::from(&packet),
                ));
            }
            SilkServerEvent::ClientLeft(id) => {
                debug!("{id:?} left");
                world_state.clients.remove(id);
                let packet = PaintingDemoPayload::Chat {
                    from: format!("{:?}", world_state.server_id.unwrap()),
                    message: format!("Goodbye {id:?}"),
                };
                event_wtr.send(SilkBroadcastEvent::ReliableSendAll(
                    SilkPayload::from(&packet),
                ));
            }
            // Message comes from Client
            SilkServerEvent::Message((id, packet)) => {
                event_wtr.send(SilkBroadcastEvent::ReliableSendAllExcept((
                    *id,
                    SilkPayload::Message(packet.clone()),
                )))
            }
            SilkServerEvent::IdAssigned(id) => {
                world_state.server_id.replace(*id);
                info!("I am {id:?}")
            }
        }
    }
    event_rdr.clear();
}
