use std::time::Duration;

use bevy::{log::LogPlugin, prelude::*, time::common_conditions::on_timer};
use bevy_rtc::server::{
    AddProtocolExt, NetworkReader, NetworkWriter, RtcServerEvent,
    RtcServerPlugin, RtcState,
};
use protocol::{ChatPayload, DrawLinePayload};

fn main() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(LogPlugin::default())
        .add_plugins(RtcServerPlugin { port: 3536 })
        .add_bounded_protocol::<ChatPayload>(2)
        .add_bounded_protocol::<DrawLinePayload>(2)
        .add_systems(
            Update,
            (
                send_draw_points,
                send_chats,
                print_events,
                print_latencies.run_if(on_timer(Duration::from_secs(1))),
            ),
        )
        .run();
}

// redirect draw points from clients to other clients
fn send_draw_points(
    mut draw_read: NetworkReader<DrawLinePayload>,
    mut draw_send: NetworkWriter<DrawLinePayload>,
) {
    for (peer, draw) in draw_read.read() {
        draw_send.unreliable_to_all_except(peer, draw);
    }
}

// redirect chat from clients to other clients
fn send_chats(
    mut chat_read: NetworkReader<ChatPayload>,
    mut chat_send: NetworkWriter<ChatPayload>,
) {
    for (peer, chat) in chat_read.read() {
        chat_send.reliable_to_all_except(peer, chat);
    }
}

fn print_events(mut event_rdr: EventReader<RtcServerEvent>) {
    for ev in event_rdr.read() {
        match ev {
            RtcServerEvent::ClientJoined(id) => {
                info!("Client joined: {id}");
            }
            RtcServerEvent::ClientLeft(id) => {
                info!("Client left: {id}");
            }
            RtcServerEvent::IdAssigned(id) => {
                info!("Server ready as {id}");
            }
        }
    }
}

fn print_latencies(state: Res<RtcState>) {
    for ((peer, latency), (_peer, smoothed)) in
        state.iter_latencies().zip(state.iter_smoothed_latencies())
    {
        info!("Latency to {peer}: {latency:.0?} (smoothed = {smoothed:.0?})");
    }
}
