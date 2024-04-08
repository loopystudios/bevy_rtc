use bevy::{log::LogPlugin, prelude::*, time::common_conditions::on_timer};
use bevy_rtc::prelude::*;
use protocol::PingPayload;
use std::time::Duration;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(LogPlugin::default())
        .add_plugins(RtcClientPlugin)
        .add_client_rw_protocol::<PingPayload>(1)
        .add_systems(
            OnEnter(RtcClientStatus::Disconnected), // Automatically-reconnect
            |mut connection_requests: EventWriter<RtcClientRequestEvent>| {
                connection_requests.send(RtcClientRequestEvent::Connect {
                    addr: "ws://127.0.0.1:3536".to_string(),
                });
            },
        )
        .add_systems(
            Update,
            {
                |mut client: RtcClient<PingPayload>| {
                    client.reliable_to_host(PingPayload::Ping);
                    info!("Sent ping...")
                }
            }
            .run_if(
                on_timer(Duration::from_secs(1)).and_then(in_state(RtcClientStatus::Connected)),
            ),
        )
        .add_systems(Update, |mut reader: RtcClient<PingPayload>| {
            for payload in reader.read() {
                if let PingPayload::Pong = payload {
                    info!("...Received pong!");
                }
            }
        })
        .run();
}
