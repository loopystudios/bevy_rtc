use bevy::{log::LogPlugin, prelude::*, time::common_conditions::on_timer};
use bevy_rtc::prelude::*;
use protocol::{PingPayload, PongPayload};
use std::time::Duration;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(LogPlugin::default())
        .add_plugins(RtcClientPlugin)
        .add_client_wo_protocol::<PingPayload>()
        .add_client_ro_protocol::<PongPayload>(1)
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
                |mut writer: RtcClient<PingPayload>| {
                    writer.reliable_to_host(PingPayload);
                    info!("Sent ping...")
                }
            }
            .run_if(
                on_timer(Duration::from_secs(1)).and_then(in_state(RtcClientStatus::Connected)),
            ),
        )
        .add_systems(Update, |mut reader: RtcClient<PongPayload>| {
            for _pong in reader.read() {
                info!("...Received pong!");
            }
        })
        .run();
}
