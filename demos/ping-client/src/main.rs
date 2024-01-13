use std::time::Duration;

use bevy::{log::LogPlugin, prelude::*, time::common_conditions::on_timer};
use bevy_rtc::client::{
    AddProtocolExt, ConnectionRequest, NetworkReader, NetworkWriter,
    RtcClientPlugin, RtcClientStatus,
};
use protocol::PingPayload;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(LogPlugin::default())
        .add_plugins(RtcClientPlugin)
        .add_bounded_protocol::<PingPayload>(1)
        .add_systems(
            OnEnter(RtcClientStatus::Disconnected), // Automatically-reconnect
            |mut connection_requests: EventWriter<ConnectionRequest>| {
                connection_requests.send(ConnectionRequest::Connect {
                    addr: "ws://127.0.0.1:3536".to_string(),
                });
            },
        )
        .add_systems(
            Update,
            {
                |mut writer: NetworkWriter<PingPayload>| {
                    writer.reliable_to_host(PingPayload::Ping);
                    info!("Sent ping..")
                }
            }
            .run_if(
                on_timer(Duration::from_secs(1)).and_then(
                    state_exists_and_equals(RtcClientStatus::Connected),
                ),
            ),
        )
        .add_systems(Update, |mut reader: NetworkReader<PingPayload>| {
            for payload in reader.read() {
                if let PingPayload::Pong = payload {
                    info!("..Received pong!");
                }
            }
        })
        .run();
}
