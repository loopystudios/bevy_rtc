use bevy::{log::LogPlugin, prelude::*, utils::HashSet};
use silk_common::demo_packets::{Chat, DrawPointMessage};
use silk_common::router::{NetworkReader, NetworkWriter};
use silk_common::schedule::SilkSchedule;
use silk_common::{bevy_matchbox::prelude::PeerId, ConnectionAddr};
use silk_common::{AddNetworkMessage, SilkSocketEvent, SilkStage};
use silk_server::SilkServerPlugin;

#[derive(Resource, Debug, Default, Clone)]
struct ServerState {
    server_id: Option<PeerId>,
    clients: HashSet<PeerId>,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugin(LogPlugin {
            filter: "warn,silk_server=error,silk_signaler=debug,painting_server=debug,wgpu_core=warn,wgpu_hal=warn,matchbox_socket=warn"
                .into(),
            level: bevy::log::Level::DEBUG,
        })
        .add_plugin(SilkServerPlugin {
            signaler_addr: ConnectionAddr::Local { port: 3536 },
            tick_rate: 1.0,
        })
        .add_system(
            handle_events
                .in_base_set(SilkStage::WriteOut)
                .in_schedule(SilkSchedule),
        )
        .add_network_message::<Chat>()
        .add_network_message::<DrawPointMessage>()
        .add_system(network_query)
        .add_system(read_chat)
        .insert_resource(ServerState::default())
        .add_startup_system(|| info!("Connecting..."))
        .run();
}

fn network_query(mut query: NetworkReader<DrawPointMessage>) {
    for test_message in query.iter() {
        error!("network queried {:?}", test_message);
    }
}

// read chats from clients
fn read_chat(
    mut chat_read: NetworkReader<Chat>,
    mut chat_send: NetworkWriter<Chat>,
) {
    for (peer, chat) in chat_read.iter() {
        error!("got chat {:?}", chat);
        chat_send.reliable_to_all_except(*peer, chat);
    }
}

fn handle_events(
    mut event_rdr: EventReader<SilkSocketEvent>,
    mut world_state: ResMut<ServerState>,
) {
    while let Some(ev) = event_rdr.iter().next() {
        match ev {
            SilkSocketEvent::ClientJoined(id) => {
                world_state.clients.insert(*id);
                debug!("{id:?} joined");
                // let packet = PaintingDemoPayload::Chat {
                //     from: format!("{:?}", world_state.server_id.unwrap()),
                //     message: format!("Hello {id:?}"),
                // };
                // event_wtr
                //     .send(SilkBroadcastEvent::ReliableSendAll(packet.into()));
            }
            SilkSocketEvent::ClientLeft(id) => {
                debug!("{id:?} left");
                world_state.clients.remove(id);
                // let packet = PaintingDemoPayload::Chat {
                //     from: format!("{:?}", world_state.server_id.unwrap()),
                //     message: format!("Goodbye {id:?}"),
                // };
                // event_wtr
                //     .send(SilkBroadcastEvent::ReliableSendAll(packet.into()));
            }
            // // Message comes from Client
            // SilkEvent::Message((id, packet)) => {
            //     let packet: Packet = packet.clone();
            //     let protocol_message =
            //         PaintingDemoPayload::from(packet.clone());
            //     for peer in world_state.clients.iter().filter(|p| p != &id) {
            //         event_wtr.send(SilkBroadcastEvent::ReliableSend((
            //             *peer,
            //             protocol_message.clone().into(),
            //         )));
            //     }
            // }
            SilkSocketEvent::IdAssigned(id) => {
                world_state.server_id.replace(*id);
                info!("I am {id:?}")
            }
            _ => {}
        }
    }
    event_rdr.clear();
}
