use std::sync::Arc;

use axum::{
    extract::{State, WebSocketUpgrade},
    response::Response,
    routing::get,
    Router,
};
use chatter_protocol::ChatterMessage;
use config::Config;
use futures_util::{SinkExt, StreamExt};
use secure_comms::Acceptor;
use tokio_rustls::rustls::ServerConfig;

fn server_config() -> Arc<ServerConfig> {
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_client_cert_verifier(todo!())
        .with_single_cert(todo!(), todo!()).unwrap();

	Arc::new(config)
}

#[derive(Clone)]
struct AppState {
    acceptor: Arc<Acceptor>,
}

async fn chatter(State(state): State<AppState>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(|ws| async move {
        tokio::spawn(async move {
            let (mut s, name) = state.acceptor.accept_with_server_name(ws).await.unwrap();
            println!("CONNECTED TO: {name:?}");
            while let Some(Ok(data)) = s.next().await {
                match data {
                    ChatterMessage::Ping(x) => {
                        s.send(ChatterMessage::Pong(x)).await.unwrap();
                    }
                    _ => todo!(),
                }
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
        });

    // run it with hyper on localhost:3000
    axum::Server::bind(&config.node.addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
