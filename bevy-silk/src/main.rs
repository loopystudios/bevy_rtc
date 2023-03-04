use bevy::{
    log::LogPlugin, prelude::*, tasks::IoTaskPool, time::FixedTimestep,
};
use matchbox_socket::WebRtcSocket;
use silk_common::SocketConfig;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    Connecting,
    InGame,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(LogPlugin {
                filter:
                    "info,wgpu_core=warn,wgpu_hal=warn,matchbox_socket=debug"
                        .into(),
                level: bevy::log::Level::DEBUG,
            })
            .set(WindowPlugin {
                window: WindowDescriptor {
                    fit_canvas_to_parent: true, // behave on wasm
                    ..default()
                },
                ..default()
            }),
    )
    .add_state(AppState::Connecting)
    .add_system_set(
        // Attempt to reconnect every 5 seconds
        SystemSet::new()
            .with_system(connect_socket)
            .with_run_criteria(FixedTimestep::step(5.0)),
    )
    .add_system_set(
        SystemSet::on_enter(AppState::Connecting).with_system(on_connecting),
    )
    .add_system_set(
        SystemSet::on_update(AppState::Connecting).with_system(poll_socket),
    )
    .add_system_set(
        SystemSet::on_enter(AppState::InGame).with_system(on_connected),
    )
    .add_system_set(
        SystemSet::on_update(AppState::InGame).with_system(poll_socket),
    )
    .add_startup_system(setup_cam)
    .run();
}

fn setup_cam(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

#[derive(Default, Resource)]
struct SocketResource(pub Option<WebRtcSocket>);

fn connect_socket(
    mut commands: Commands,
    socket_res: Option<ResMut<SocketResource>>,
    app_state: ResMut<State<AppState>>,
) {
    if let AppState::Connecting = app_state.current() {
        if let Some(mut socket_res) = socket_res {
            socket_res.0.take();
        }
        let config = SocketConfig::LocalClient { port: 3536 }.get();
        info!("connecting to matchbox server");
        let (socket, message_loop) = WebRtcSocket::new_with_config(config);

        // The message loop needs to be awaited, or nothing will happen.
        // We do this here using bevy's task system.
        let task_pool = IoTaskPool::get();
        task_pool.spawn(message_loop).detach();

        commands.insert_resource(SocketResource(Some(socket)));
    }
}

fn on_connecting(mut commands: Commands) {
    commands.insert_resource(ClearColor(Color::RED));
}

fn on_connected(mut commands: Commands) {
    commands.insert_resource(ClearColor(Color::GREEN));
}

fn poll_socket(
    mut app_state: ResMut<State<AppState>>,
    mut socket_res: Option<ResMut<SocketResource>>,
) {
    if let Some(socket) = socket_res.as_mut().and_then(|s| s.0.as_mut()) {
        // Process event queue that came through the socket receiver to
        // ensure
        socket.update_peers();
        // Count connected peers
        let connected_peers = socket.connected_peers().count();

        if connected_peers == 0 {
            // Not connected!
            if let AppState::InGame = app_state.current() {
                warn!("DISCONNECTED!!!");
                app_state.set(AppState::Connecting).expect(
                    "Tried to go back to connecting while already connecting",
                );
            }
        } else {
            // Host connected!
            if let AppState::Connecting = app_state.current() {
                for peer in socket.connected_peers() {
                    info!("I am connected to {peer}");
                }
                app_state
                    .set(AppState::InGame)
                    .expect("Tried to go in-game while already in-game");
            }
        }
    }
}
