use bevy::{log::LogPlugin, prelude::*};
use bevy_silk::server::{
    AddNetworkMessageExt, NetworkReader, NetworkWriter, SilkServerEvent,
    SilkServerPlugin,
};
use protocol::{ChatPayload, DrawLinePayload};

fn main() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(LogPlugin::default())
        .add_plugins(SilkServerPlugin { port: 3536 })
        .add_network_message::<ChatPayload>()
        .add_network_message::<DrawLinePayload>()
        .add_systems(Update, (player_auth, send_draw_points, send_chats))
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

fn player_auth(mut event_rdr: EventReader<SilkServerEvent>) {
    for ev in event_rdr.read() {
        match ev {
            SilkServerEvent::ClientJoined(id) => {
                info!("{id} joined");
            }
            SilkServerEvent::ClientLeft(id) => {
                info!("{id} left");
            }
            _ => {}
        }
    }
}
