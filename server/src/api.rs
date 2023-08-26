use axum::{
    extract::{State, WebSocketUpgrade},
    response::Response,
    routing::get,
    Router,
};
use chatter_protocol::ChatterMessage;
use tracing::{error, info};

use crate::{
    node_manager::{
        event_triggers::{EventHandlers, FromErrors},
        Connection, ConnectionError,
    },
    AppState,
};

async fn chatter<Ev>(State(state): State<AppState<Ev>>, ws: WebSocketUpgrade) -> Response
where
    Ev: EventHandlers + Send + Sync + 'static,
    ConnectionError: FromErrors<Ev>,
{
    ws.on_upgrade(|ws| async move {
        tokio::spawn(async move {
            let (s, name) = state.acceptor.accept_with_server_name(ws).await.unwrap();
            if let Some(name) = name {
                info!("Connected to {}", name);
                let connection =
                    Connection::accepted(s, state.config.clone(), state.ev.clone(), name.clone());
                let connected = state
                    .node_manager
                    .read()
                    .await
                    .connected()
                    .map(|s| s.to_string())
                    .collect();
                let msg = ChatterMessage::Hello {
                    config: state.config.general.clone(),
                    priority: state.config.node.priority,
                    connected,
                };
                connection.send(msg).await.unwrap();
                state.node_manager.write().await.up(name, connection)
            } else {
                error!("NO NAME IN CERTIFICATE PROVIDED");
            }
        });
    })
}

fn info<Ev>() -> Router<AppState<Ev>>
where
    Ev: Send + Sync + EventHandlers + 'static,
    ConnectionError: FromErrors<Ev>,
{
    todo!()
}

pub(crate) fn api<Ev>() -> Router<AppState<Ev>>
where
    Ev: Send + Sync + EventHandlers + 'static,
    ConnectionError: FromErrors<Ev>,
{
    Router::new()
        .route("/chatter", get(chatter))
        .nest("/info", info())
}
