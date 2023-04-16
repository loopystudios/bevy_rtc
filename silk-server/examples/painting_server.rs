use bevy::{log::LogPlugin, prelude::*, utils::HashSet};
use silk_common::demo_packets::{Chat, DrawPoint};
use silk_common::events::SilkServerEvent;
use silk_common::packets::auth::SilkLoginAcceptedPayload;
use silk_common::router::{NetworkReader, NetworkWriter};
use silk_common::schedule::SilkSchedule;
use silk_common::{bevy_matchbox::prelude::PeerId, ConnectionAddr};
use silk_common::{AddNetworkMessageExt, SilkStage};
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
                .in_base_set(SilkStage::WriteOut)
                .in_schedule(SilkSchedule),
        )
        .add_network_message::<Chat>()
        .add_network_message::<DrawPoint>()
        .add_system(handle_draw_points)
        .add_system(handle_chats)
        .insert_resource(ServerState::default())
        .add_startup_system(|| info!("Connecting..."))
        .run();
}

// redirect draw points from clients to other clients
fn handle_draw_points(
    mut draw_read: NetworkReader<DrawPoint>,
    mut draw_send: NetworkWriter<DrawPoint>,
) {
    for (peer, draw) in draw_read.iter() {
        draw_send.unreliable_to_all_except(*peer, draw);
    }
}

// redirect chat from clients to other clients
fn handle_chats(
    mut chat_read: NetworkReader<Chat>,
    mut chat_send: NetworkWriter<Chat>,
) {
    for (peer, chat) in chat_read.iter() {
        chat_send.reliable_to_all_except(*peer, chat);
    }
}

fn handle_events(
    mut guest_count: Local<u16>,
    mut accept_wtr: NetworkWriter<SilkLoginAcceptedPayload>,
    mut event_rdr: EventReader<SilkServerEvent>,
    mut world_state: ResMut<ServerState>,
) {
    while let Some(ev) = event_rdr.iter().next() {
        match ev {
            SilkServerEvent::GuestLoginRequest { peer_id, .. }
            | SilkServerEvent::LoginRequest { peer_id, .. } => {
                debug!("{peer_id:?} joined");

                *guest_count += 1;
                let username = format!("Guest-{}", *guest_count);

                debug!("{peer_id:?} : {username} joined");
                world_state.clients.insert(*peer_id);
                accept_wtr.reliable_to_peer(
                    *peer_id,
                    &SilkLoginAcceptedPayload { username },
                );
            }
            SilkServerEvent::ClientLeft(id) => {
                debug!("{id:?} left");
                world_state.clients.remove(id);
            }
            SilkServerEvent::IdAssigned(id) => {
                world_state.server_id.replace(*id);
                info!("I am {id:?}")
            }
        }
    }
    event_rdr.clear();
}
