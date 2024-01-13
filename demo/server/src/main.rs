use std::time::Duration;

use bevy::{log::LogPlugin, prelude::*};
use bevy_silk::server::{
    AddNetworkMessageExt, NetworkReader, NetworkWriter, SilkServerEvent,
    SilkServerPlugin, SilkState,
};
use protocol::{ChatPayload, DrawLinePayload};

fn main() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(LogPlugin::default())
        .add_plugins(SilkServerPlugin { port: 3536 })
        .add_network_message::<ChatPayload>()
        .add_network_message::<DrawLinePayload>()
        .add_systems(
            Update,
            (send_draw_points, send_chats, print_events, print_latencies),
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

fn print_events(mut event_rdr: EventReader<SilkServerEvent>) {
    for ev in event_rdr.read() {
        match ev {
            SilkServerEvent::ClientJoined(id) => {
                info!("Client joined: {id}");
            }
            SilkServerEvent::ClientLeft(id) => {
                info!("Client left: {id}");
            }
            SilkServerEvent::IdAssigned(id) => {
                info!("Server ready as {id}");
            }
        }
    }
}

fn print_latencies(
    state: Res<SilkState>,
    time: Res<Time>,
    mut throttle: Local<Option<Timer>>,
) {
    let timer = throttle.get_or_insert(Timer::new(
        Duration::from_millis(100),
        TimerMode::Repeating,
    ));
    timer.tick(time.delta());
    if timer.just_finished() {
        for ((peer, latency), (_peer, smoothed)) in
            state.iter_latencies().zip(state.iter_smoothed_latencies())
        {
            info!(
                "Latency to {peer}: {latency:.0?} (smoothed = {smoothed:.0?})"
            );
        }
    }
}
