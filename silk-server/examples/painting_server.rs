use bevy::{log::LogPlugin, prelude::*, utils::HashSet};
use silk_common::bevy_matchbox::matchbox_socket::Packet;
use silk_common::{
    bevy_matchbox::prelude::PeerId, demo_packets::PaintingDemoPayload,
    ConnectionAddr,
};
use silk_server::{
    events::{SilkBroadcastEvent, SilkServerEvent},
    SilkServerPlugin,
};
use silk_server::{SilkStage, SilkStageSchedule};

#[derive(Resource, Debug, Default, Clone)]
struct ServerState {
    server_id: Option<PeerId>,
    clients: HashSet<PeerId>,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugin(LogPlugin {
            filter: "warn,silk_server=debug,silk_signaler=debug,painting_server=debug,wgpu_core=warn,wgpu_hal=warn,matchbox_socket=warn"
                .into(),
            level: bevy::log::Level::DEBUG,
        })
        .add_plugin(SilkServerPlugin {
            signaler_addr: ConnectionAddr::Local { port: 3536 },
            tick_rate: 1.0,
        })
        .add_system(
            handle_events
                .in_base_set(SilkStage::WriteSocket)
                .in_schedule(SilkStageSchedule),
        )
        .insert_resource(ServerState::default())
        .add_startup_system(|| info!("Connecting..."))
        .run();
}

fn handle_events(
    mut event_rdr: EventReader<SilkServerEvent>,
    mut event_wtr: EventWriter<SilkBroadcastEvent>,
    mut world_state: ResMut<ServerState>,
) {
    while let Some(ev) = event_rdr.iter().next() {
        match ev {
            SilkServerEvent::ClientJoined(id) => {
                world_state.clients.insert(*id);
                debug!("{id:?} joined");
                let packet = PaintingDemoPayload::Chat {
                    from: format!("{:?}", world_state.server_id.unwrap()),
                    message: format!("Hello {id:?}"),
                };
                event_wtr
                    .send(SilkBroadcastEvent::ReliableSendAll(packet.into()));
            }
            SilkServerEvent::ClientLeft(id) => {
                debug!("{id:?} left");
                world_state.clients.remove(id);
                let packet = PaintingDemoPayload::Chat {
                    from: format!("{:?}", world_state.server_id.unwrap()),
                    message: format!("Goodbye {id:?}"),
                };
                event_wtr
                    .send(SilkBroadcastEvent::ReliableSendAll(packet.into()));
            }
            // Message comes from Client
            SilkServerEvent::Message((id, packet)) => {
                let packet: Packet = packet.clone();
                let protocol_message =
                    PaintingDemoPayload::from(packet.clone());
                for peer in world_state.clients.iter().filter(|p| p != &id) {
                    event_wtr.send(SilkBroadcastEvent::ReliableSend((
                        *peer,
                        protocol_message.clone().into(),
                    )));
                }
            }
            SilkServerEvent::IdAssigned(id) => {
                world_state.server_id.replace(*id);
                info!("I am {id:?}")
            }
        }
    }
    event_rdr.clear();
}
