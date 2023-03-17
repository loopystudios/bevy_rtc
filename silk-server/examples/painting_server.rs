use bevy::{log::LogPlugin, prelude::*, utils::HashSet};
use matchbox_socket::{Packet, PeerId};
use silk_common::demo_packets::Payload;
use silk_server::{
    events::{SilkBroadcastEvent, SilkServerEvent},
    stages, SilkServerPlugin,
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
            filter: "error,painting_server=debug,wgpu_core=warn,wgpu_hal=warn,matchbox_socket=warn"
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
            SilkServerEvent::PeerJoined(id) => {
                world_state.clients.insert(*id);
                debug!("{id:?} joined");
                let packet = Payload::Chat {
                    from: world_state.server_id.unwrap(),
                    message: "someone joined!".to_string(),
                };
                event_wtr
                    .send(SilkBroadcastEvent::ReliableSendAll(packet.into()));
            }
            SilkServerEvent::PeerLeft(id) => {
                debug!("{id:?} left");
                world_state.clients.remove(id);
                let packet = Payload::Chat {
                    from: world_state.server_id.unwrap(),
                    message: "someone left!".to_string(),
                };
                event_wtr
                    .send(SilkBroadcastEvent::ReliableSendAll(packet.into()));
            }
            // Message comes from Client
            SilkServerEvent::Message((id, packet)) => {
                let packet: Packet = packet.clone();
                let protocol_message = Payload::from(packet.clone());
                match protocol_message {
                    Payload::Chat { from, message } => {
                        for peer in
                            world_state.clients.iter().filter(|p| p != &id)
                        {
                            event_wtr.send(SilkBroadcastEvent::ReliableSend((
                                *peer,
                                packet.clone(),
                            )));
                        }
                    }
                    Payload::DrawPoint { x1, y1, x2, y2 } => {
                        for peer in
                            world_state.clients.iter().filter(|p| p != &id)
                        {
                            event_wtr.send(SilkBroadcastEvent::ReliableSend((
                                *peer,
                                packet.clone(),
                            )));
                        }
                    }
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