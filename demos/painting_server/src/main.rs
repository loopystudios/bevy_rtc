use bevy::{log::LogPlugin, prelude::*, time::common_conditions::on_timer};
use bevy_rtc::prelude::*;
use protocol::{ChatPayload, DrawLinePayload};
use std::time::Duration;

fn main() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(LogPlugin::default())
        .add_plugins(RtcServerPlugin {
            port: 3536,
            // CAREFUL: This encoding MUST match the client encoding!
            encoding: TransportEncoding::Json,
        })
        .add_server_rw_protocol::<ChatPayload>(2)
        .add_server_rw_protocol::<DrawLinePayload>(2)
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
fn send_draw_points(mut server: RtcServer<DrawLinePayload>) {
    for (peer, draw) in server.read() {
        server.unreliable_to_all_except(peer, draw);
    }
}

// redirect chat from clients to other clients
fn send_chats(mut server: RtcServer<ChatPayload>) {
    for (peer, chat) in server.read() {
        server.reliable_to_all_except(peer, chat);
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

fn print_latencies(state: Res<RtcServerState>) {
    for ((peer, latency), (_peer, smoothed)) in
        state.iter_latencies().zip(state.iter_smoothed_latencies())
    {
        info!("Latency to {peer}: {latency:.0?} (smoothed = {smoothed:.0?})");
    }
}
