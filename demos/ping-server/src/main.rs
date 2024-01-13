use bevy::{log::LogPlugin, prelude::*};
use bevy_rtc::server::{
    AddProtocolExt, NetworkReader, NetworkWriter, RtcServerPlugin,
};
use protocol::PingPayload;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(LogPlugin::default())
        .add_plugins(RtcServerPlugin { port: 3536 })
        .add_bounded_protocol::<PingPayload>(1)
        .add_systems(
            Update,
            |mut reader: NetworkReader<PingPayload>,
             mut writer: NetworkWriter<PingPayload>| {
                for (peer_id, packet) in reader.read() {
                    if let PingPayload::Ping = packet {
                        info!("Received ping! Sending pong...");
                        writer.reliable_to_peer(peer_id, PingPayload::Pong);
                    }
                }
            },
        )
        .run();
}
