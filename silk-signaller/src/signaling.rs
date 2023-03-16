use crate::{
    error::ClientRequestError,
    glue::{Peer, PeerEvent, PeerRequest},
    state::ServerState,
};
use axum::{
    extract::{
        connect_info::ConnectInfo,
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::IntoResponse,
    Error,
};
use futures::{lock::Mutex, stream::SplitSink, StreamExt};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tracing::{error, info, warn};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) enum ClientType {
    Client,
    Host,
}

/// The handler for the HTTP request to upgrade to WebSockets.
/// This is the last point where we can extract TCP/IP metadata such as IP
/// address of the client.
pub(crate) async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(client_type): Path<ClientType>,
    State(state): State<Arc<Mutex<ServerState>>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    info!("`{addr}` connected.");
    info!("client type: {:?}", client_type);

    let is_host = matches!(client_type, ClientType::Host);

    // Finalize the upgrade process by returning upgrade callback to client
    ws.on_upgrade(move |websocket| handle_ws(websocket, state, is_host))
}

fn parse_request(
    request: Result<Message, Error>,
) -> Result<PeerRequest, ClientRequestError> {
    let request = request?;

    let request: PeerRequest = match request {
        Message::Text(text) => serde_json::from_str(&text)?,
        Message::Close(_) => return Err(ClientRequestError::Close),
        _ => return Err(ClientRequestError::UnsupportedType),
    };

    Ok(request)
}

fn spawn_sender_task(
    sender: SplitSink<WebSocket, Message>,
) -> mpsc::UnboundedSender<Result<Message, Error>> {
    let (client_sender, receiver) = mpsc::unbounded_channel();
    tokio::task::spawn(UnboundedReceiverStream::new(receiver).forward(sender));
    client_sender
}

/// One of these handlers is spawned for every web socket.
async fn handle_ws(
    websocket: WebSocket,
    state: Arc<Mutex<ServerState>>,
    is_host: bool,
) {
    let (ws_sender, mut ws_receiver) = websocket.split();
    let sender = spawn_sender_task(ws_sender);

    // Ensure host setup
    {
        let state = state.lock().await;
        if !is_host && state.host.is_none() {
            error!("client cannot connect until there's a host");
            return;
        }
        if is_host && state.host.is_some() {
            error!("there is already a host");
            return;
        }
    }

    // Generate a UUID for the user
    let peer_uuid = matchbox_socket::PeerId(uuid::Uuid::new_v4());
    {
        let mut state = state.lock().await;
        state.add_client(Peer {
            uuid: peer_uuid,
            sender: sender.clone(),
        });

        let event_text =
            serde_json::to_string(&PeerEvent::IdAssigned(peer_uuid))
                .expect("error serializing message");
        let event = Message::Text(event_text.clone());

        if let Err(e) = state.try_send(&peer_uuid, event) {
            error!("error sending to {peer_uuid:?}: {e:?}");
            return;
        } else {
            info!("{peer_uuid:?} -> {event_text}");
        };

        if is_host {
            // Set host
            state.host.replace(Peer {
                uuid: peer_uuid,
                sender: sender.clone(),
            });
            info!("SET HOST: {peer_uuid:?}");
        } else {
            // Set client
            let peer_event = PeerEvent::NewPeer(peer_uuid);
            let event_text = serde_json::to_string(&peer_event)
                .expect("error serializing message");
            let event = Message::Text(event_text.clone());

            // Tell host about this new client
            if let Err(e) = state.try_send_to_host(event) {
                error!("error sending peer {peer_uuid:?} to host: {e:?}");
                return;
            } else {
                info!("{peer_uuid:?} -> {event_text}");
            }
        }
    }

    // The state machine for the data channel established for this websocket.
    while let Some(request) = ws_receiver.next().await {
        // Parse the message
        let request = match parse_request(request) {
            Ok(request) => request,
            Err(ClientRequestError::Axum(e)) => {
                // Most likely a ConnectionReset or similar.
                error!("Axum error while receiving request: {:?}", e);
                warn!("Severing connection with {peer_uuid:?}");
                break; // give up on this peer.
            }
            Err(ClientRequestError::Close) => {
                info!("Received websocket close from {peer_uuid:?}");
                break;
            }
            Err(e) => {
                error!("Error untangling request: {:?}", e);
                continue;
            }
        };

        // Handle the message
        match request {
            PeerRequest::Signal { receiver, data } => {
                info!("<-- {peer_uuid:?}: {data:?}");
                let event = Message::Text(
                    serde_json::to_string(&PeerEvent::Signal {
                        sender: peer_uuid,
                        data,
                    })
                    .expect("error serializing message"),
                );
                let state = state.lock().await;
                if let Some(peer) = state.clients.get(&receiver) {
                    if let Err(e) = peer.sender.send(Ok(event)) {
                        error!("error sending: {e:?}");
                    }
                } else {
                    warn!("peer not found ({receiver:?}), ignoring signal");
                }
            }
            PeerRequest::KeepAlive => {
                // Do nothing. KeepAlive packets are used to protect against
                // users' browsers disconnecting idle websocket connections.
            }
        }
    }

    // Peer disconnected or otherwise ended communication.
    info!("Removing peer: {:?}", peer_uuid);
    let mut state = state.lock().await;

    if state
        .host
        .as_ref()
        .is_some_and(|host| host.uuid == peer_uuid)
    {
        // Tell each connected peer about the disconnected host.
        let peer_event = PeerEvent::PeerLeft(state.host.as_ref().unwrap().uuid);
        let event = Message::Text(
            serde_json::to_string(&peer_event)
                .expect("error serializing message"),
        );
        for peer_id in state
            .clients
            .keys()
            .filter(|id| id != &&state.host.as_ref().unwrap().uuid)
        {
            match state.try_send(peer_id, event.clone()) {
                Ok(()) => {
                    info!("Sent host peer remove to: {peer_id:?}")
                }
                Err(e) => {
                    error!("Failure sending host peer remove to {peer_id:?}: {e:?}")
                }
            }
        }
        state.reset();
    } else if let Some(removed_peer) = state.remove_client(&peer_uuid) {
        if state.host.is_some() {
            // Tell host about disconnected clent
            let peer_event = PeerEvent::PeerLeft(removed_peer.uuid);
            let event = Message::Text(
                serde_json::to_string(&peer_event)
                    .expect("error serializing message"),
            );
            match state.try_send_to_host(event) {
                Ok(()) => {
                    info!(
                        "Notified host of peer remove: {:?}",
                        &removed_peer.uuid
                    )
                }
                Err(e) => {
                    error!("Failure sending peer remove to host: {e:?}")
                }
            }
        }
    }
}
