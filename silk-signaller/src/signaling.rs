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
) -> Result<PeerRequest<serde_json::Value>, ClientRequestError> {
    let request = request?;

    let request: PeerRequest<serde_json::Value> = match request {
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
    let mut peer_uuid = None;

    // The state machine for the data channel established for this websocket.
    while let Some(request) = ws_receiver.next().await {
        // Parse the message
        let request = match parse_request(request) {
            Ok(request) => request,
            Err(ClientRequestError::Axum(e)) => {
                // Most likely a ConnectionReset or similar.
                error!("Axum error while receiving request: {:?}", e);
                if let Some(ref peer_uuid) = peer_uuid {
                    warn!("Severing connection with {peer_uuid}")
                }
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

        info!("{:?} <- {:?}", peer_uuid, request);

        match request {
            PeerRequest::Uuid(id) => {
                let mut state = state.lock().await;
                if peer_uuid.is_some() {
                    error!("client set uuid more than once");
                    continue;
                }

                if !is_host && state.host.is_none() {
                    error!("client cannot connect until there's a host");
                    break;
                }
                if is_host && state.host.is_some() {
                    error!("there is already a host");
                    break;
                }

                // Add peer
                peer_uuid.replace(id.clone());
                state.add_client(Peer {
                    uuid: id.clone(),
                    sender: sender.clone(),
                });

                if is_host {
                    // Set host
                    state.host.replace(Peer {
                        uuid: id.clone(),
                        sender: sender.clone(),
                    });
                    info!("SET HOST: {id}");
                } else {
                    // Set client
                    let peer_event =
                        PeerEvent::<serde_json::Value>::NewPeer(id.clone());
                    let event_text = serde_json::to_string(&peer_event)
                        .expect("error serializing message");
                    let event = Message::Text(event_text.clone());

                    // Tell host about this new client
                    if let Err(e) = state.try_send_to_host(event.clone()) {
                        error!("error sending peer {id} to host: {e:?}");
                    } else {
                        info!("{:?} -> {:?}", id, event_text);
                    }
                }
            }
            PeerRequest::Signal { receiver, data } => {
                let sender = match peer_uuid.clone() {
                    Some(sender) => sender,
                    None => {
                        error!("client is trying signal before sending uuid");
                        continue;
                    }
                };
                let event = Message::Text(
                    serde_json::to_string(&PeerEvent::Signal { sender, data })
                        .expect("error serializing message"),
                );
                let state = state.lock().await;
                if let Some(peer) = state.clients.get(&receiver) {
                    if let Err(e) = peer.sender.send(Ok(event)) {
                        error!("error sending: {:?}", e);
                    }
                } else {
                    warn!("peer not found ({receiver}), ignoring signal");
                }
            }
            PeerRequest::KeepAlive => {}
        }
    }

    // Peer disconnected or otherwise ended communication.
    if let Some(uuid) = peer_uuid {
        info!("Removing peer: {:?}", uuid);
        let mut state = state.lock().await;

        if state.host.as_ref().is_some_and(|host| host.uuid == uuid) {
            // Tell each connected peer about the disconnected host.
            let peer_event = PeerEvent::<serde_json::Value>::PeerLeft(
                state.host.as_ref().unwrap().uuid.clone(),
            );
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
                        info!("Sent host peer remove to: {:?}", peer_id)
                    }
                    Err(e) => {
                        error!("Failure sending peer remove: {e:?}")
                    }
                }
            }
            state.host.take();
        } else if let Some(removed_peer) = state.remove_client(&uuid) {
            // Host must exist
            // Tell host about disconnected clent
            let peer_event = PeerEvent::<serde_json::Value>::PeerLeft(
                removed_peer.uuid.clone(),
            );
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
