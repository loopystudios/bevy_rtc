use bevy::{log::LogPlugin, prelude::*};
use bevy_silk::{
    packets::auth::SilkLoginResponsePayload,
    schedule::{SilkSchedule, SilkSet},
    server::{
        AddNetworkMessageExt, NetworkReader, NetworkWriter, SilkServerEvent,
        SilkServerPlugin,
    },
};
use protocol::{ChatPayload, DrawLinePayload};

fn main() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(LogPlugin::default())
        .add_plugins(SilkServerPlugin {
            port: 3536,
            tick_rate: 60.0,
        })
        .add_network_message::<ChatPayload>()
        .add_network_message::<DrawLinePayload>()
        .add_systems(Update, handle_events)
        .add_systems(SilkSchedule, send_draw_points.in_set(SilkSet::Update))
        .add_systems(SilkSchedule, send_chats.in_set(SilkSet::Update))
        .run();
}

// redirect draw points from clients to other clients
fn send_draw_points(
    mut draw_read: NetworkReader<DrawLinePayload>,
    mut draw_send: NetworkWriter<DrawLinePayload>,
) {
    for (peer, draw) in draw_read.iter() {
        draw_send.unreliable_to_all_except(*peer, draw.clone());
    }
}

// redirect chat from clients to other clients
fn send_chats(
    mut chat_read: NetworkReader<ChatPayload>,
    mut chat_send: NetworkWriter<ChatPayload>,
) {
    for (peer, chat) in chat_read.iter() {
        chat_send.reliable_to_all_except(*peer, chat.clone());
    }
}

fn handle_events(
    mut guest_count: Local<u16>,
    mut accept_wtr: NetworkWriter<SilkLoginResponsePayload>,
    mut event_rdr: EventReader<SilkServerEvent>,
) {
    for ev in event_rdr.read() {
        debug!("event: {ev:?}");
        match ev {
            SilkServerEvent::GuestLoginRequest { peer_id, .. }
            | SilkServerEvent::LoginRequest { peer_id, .. } => {
                *guest_count += 1;
                let username = format!("Guest-{}", *guest_count);
                info!("{peer_id} : {username} joined");
                // Accept all users
                accept_wtr.reliable_to_peer(
                    *peer_id,
                    SilkLoginResponsePayload::Accepted { username },
                );
            }
            SilkServerEvent::ClientLeft(id) => {
                info!("{id} left");
            }
            _ => {}
        }
    }
    event_rdr.clear();
}
