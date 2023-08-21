use std::sync::Arc;

use axum::{
    extract::{State, WebSocketUpgrade},
    response::Response,
    routing::get,
    Router,
};
use config::Config;
use node_manager::{Connection, NodeManager};
use secure_comms::Acceptor;
use tokio::sync::RwLock;
use tokio_rustls::rustls::ServerConfig;

pub mod node_manager;

fn server_config() -> Arc<ServerConfig> {
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_client_cert_verifier(todo!())
        .with_single_cert(todo!(), todo!())
        .unwrap();

    Arc::new(config)
}

#[derive(Clone)]
struct AppState {
    acceptor: Arc<Acceptor>,
    node_manager: Arc<RwLock<NodeManager>>,
}

async fn chatter(State(state): State<AppState>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(|ws| async move {
        tokio::spawn(async move {
            let (s, name) = state.acceptor.accept_with_server_name(ws).await.unwrap();
            if let Some(name) = name {
                state
                    .node_manager
                    .write()
                    .await
                    .up(name, Connection::accepted(s))
            }
        });
    })
}

pub async fn start(config: Config) {
    let api = Router::new().route("/chatter", get(chatter));

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .nest("/api", api)
        .with_state(AppState {
            acceptor: Arc::new(Acceptor::from(server_config())),
            node_manager: Default::default(),
        });

    // run it with hyper on localhost:3000
    axum::Server::bind(&config.node.addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
