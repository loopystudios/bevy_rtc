use async_compat::CompatExt;
use bevy::{prelude::*, tasks::IoTaskPool};
use matchbox_signaling::SignalingServer;
use std::net::Ipv4Addr;

/// The signaler abstraction
pub struct SilkSignalerPlugin {
    /// The port to broadcast on
    pub port: u16,
}

impl Plugin for SilkSignalerPlugin {
    fn build(&self, _app: &mut App) {
        let task_pool = IoTaskPool::get();
        task_pool
            .spawn(Box::pin(start_signaler(self.port).compat()))
            .detach();
    }
}

async fn start_signaler(port: u16) {
    let server =
        SignalingServer::client_server_builder((Ipv4Addr::UNSPECIFIED, port))
            .on_connection_request(|connection| {
                debug!("Connecting: {connection:?}");
                Ok(true) // Allow all connections
            })
            .cors()
            .build();

    server.serve().await.unwrap();
}
