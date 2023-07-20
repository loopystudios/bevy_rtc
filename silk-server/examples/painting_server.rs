use bevy::{log::LogPlugin, prelude::*, utils::HashSet};
use silk_common::{
    bevy_matchbox::prelude::PeerId,
    demo_packets::{Chat, DrawPoint},
    events::SilkServerEvent,
    packets::auth::SilkLoginResponsePayload,
    schedule::SilkSchedule,
    stage::SilkStage,
    ConnectionAddr,
};
use silk_server::{
    AddNetworkMessageExt, NetworkReader, NetworkWriter, SilkServerPlugin,
};

#[derive(Resource, Debug, Default, Clone)]
struct ServerState {
    server_id: Option<PeerId>,
    clients: HashSet<PeerId>,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(LogPlugin {
            filter: "warn,painting_server=debug,silk=trace,wgpu_core=warn,wgpu_hal=warn,matchbox_socket=warn"
                .into(),
            level: bevy::log::Level::DEBUG,
        })
        .add_plugins(SilkServerPlugin {
            signaler_addr: ConnectionAddr::Local { port: 3536 },
            tick_rate: 1.0,
        })
        .add_systems(
            SilkSchedule,
            handle_events
                .in_set(SilkStage::SilkEvents),
        )
        .add_network_message::<Chat>()
        .add_network_message::<DrawPoint>()
        .add_systems(
            SilkSchedule,
            send_draw_points
                .in_set(SilkStage::NetworkWrite)
        )
        .add_systems(
            SilkSchedule,
            send_chats
                .in_set(SilkStage::NetworkWrite)
        )
        .insert_resource(ServerState::default())
        .add_systems(Startup, || info!("Connecting..."))
        .run();
}

// redirect draw points from clients to other clients
fn send_draw_points(
    mut draw_read: NetworkReader<DrawPoint>,
    mut draw_send: NetworkWriter<DrawPoint>,
) {
    for (peer, draw) in draw_read.iter() {
        draw_send.unreliable_to_all_except(*peer, draw.clone());
    }
}

// redirect chat from clients to other clients
fn send_chats(
    mut chat_read: NetworkReader<Chat>,
    mut chat_send: NetworkWriter<Chat>,
) {
    for (peer, chat) in chat_read.iter() {
        chat_send.reliable_to_all_except(*peer, chat.clone());
    }
}

fn handle_events(
    mut guest_count: Local<u16>,
    mut accept_wtr: NetworkWriter<SilkLoginResponsePayload>,
    mut event_rdr: EventReader<SilkServerEvent>,
    mut world_state: ResMut<ServerState>,
) {
    for ev in event_rdr.iter() {
        debug!("event: {ev:?}");
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
                    SilkLoginResponsePayload::Accepted { username },
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
