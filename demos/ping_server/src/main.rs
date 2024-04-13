use bevy::{log::LogPlugin, prelude::*};
use bevy_rtc::prelude::*;
use protocol::{PingPayload, PongPayload};

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(LogPlugin::default())
        .add_plugins(RtcServerPlugin { port: 3536 })
        .add_server_ro_protocol::<PingPayload>(1)
        .add_server_wo_protocol::<PongPayload>()
        .add_systems(
            Update,
            |mut reader: RtcServer<PingPayload>, mut writer: RtcServer<PongPayload>| {
                for (peer_id, _ping) in reader.read() {
                    info!("Received ping! Sending pong...");
                    writer.reliable_to_peer(peer_id, PongPayload);
                }
            },
        )
        .run();
}
