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
            .on_id_assignment(|(socket, id)| debug!("{socket} received {id:?}"))
            .on_host_connected(|id| info!("Host joined: {id:?}"))
            .on_host_disconnected(|id| info!("Host left: {id:?}"))
            .on_client_connected(|id| info!("Client joined: {id:?}"))
            .on_client_disconnected(|id| info!("Client left: {id:?}"))
            .cors()
            .build();

    server.serve().await.unwrap();
}
