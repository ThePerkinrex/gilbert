use std::collections::HashMap;

use axum::{
    extract::{State, WebSocketUpgrade, Path},
    response::Response,
    routing::get,
    Router, Json,
};
use chatter_protocol::ChatterMessage;
use config::Param;
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

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
enum NodeStatus {
    Connected, Disconnected
}

async fn nodes<Ev>(State(state): State<AppState<Ev>>) -> Json<HashMap<String, NodeStatus>>
where
    Ev: Send + Sync + EventHandlers + 'static,
    ConnectionError: FromErrors<Ev>,
{
    Json(state.node_manager.read().await.nodes().map(|(name, status)| (name.to_string(), match status {
        crate::node_manager::NodeStatus::Down | crate::node_manager::NodeStatus::Unknown => NodeStatus::Disconnected,
        crate::node_manager::NodeStatus::Up(_) => NodeStatus::Connected,
    })).collect())
}

async fn jobs<Ev>(State(state): State<AppState<Ev>>) -> Json<Vec<String>>
    where
        Ev: Send + Sync + EventHandlers + 'static,
        ConnectionError: FromErrors<Ev>,
    {
        Json(state.config.general.tasks.keys().cloned().collect())
    }

async fn job<Ev>(State(state): State<AppState<Ev>>, Path(name): Path<String>) -> Json<()>
    where
        Ev: Send + Sync + EventHandlers + 'static,
        ConnectionError: FromErrors<Ev>,
    {
        todo!();
        Json(())
    }

#[allow(clippy::redundant_pub_crate)]
pub(crate) fn api<Ev>() -> Router<AppState<Ev>>
where
    Ev: Send + Sync + EventHandlers + 'static,
    ConnectionError: FromErrors<Ev>,
{
    Router::new()
        .route("/chatter", get(chatter))
        .route("/nodes", get(nodes))
        .route("/jobs", get(jobs))
}
