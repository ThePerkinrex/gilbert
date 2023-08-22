use std::sync::Arc;

use axum::{
    extract::{State, WebSocketUpgrade},
    response::Response,
    routing::get,
    Router,
};
use cache::{CacheError, CertificatesCache};
use chatter_protocol::ChatterMessage;
use config::Config;
use node_manager::{Connection, NodeManager};
use secure_comms::{connector, Acceptor};
use tokio::sync::RwLock;
use tokio_rustls::rustls::{
    server::AllowAnyAuthenticatedClient, ClientConfig, RootCertStore, ServerConfig, ServerName,
};

mod cache;
mod node_manager;

fn server_config(
    conf: &Config,
    cache: &CertificatesCache,
) -> Result<Arc<ServerConfig>, CacheError> {
    let mut rcs = RootCertStore::empty();
    for cert in cache.get_ca(conf)? {
        rcs.add(cert)?;
    }
    let client_cert_verifier = AllowAnyAuthenticatedClient::new(rcs).boxed();
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_client_cert_verifier(client_cert_verifier)
        .with_single_cert(cache.get_cert(conf)?, cache.get_key(conf)?)?;
    Ok(Arc::new(config))
}

fn client_config(
    conf: &Config,
    cache: &CertificatesCache,
) -> Result<Arc<ClientConfig>, CacheError> {
    let mut rcs = RootCertStore::empty();
    for cert in cache.get_ca(conf)? {
        rcs.add(cert)?;
    }
    let config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(rcs)
        .with_client_auth_cert(cache.get_cert(conf)?, cache.get_key(conf)?)?;
    Ok(Arc::new(config))
}

#[derive(Clone)]
struct AppState {
    acceptor: Arc<Acceptor>,
    node_manager: Arc<RwLock<NodeManager>>,
    // client_config: Arc<ClientConfig>,
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
    let cache = CertificatesCache::default();
    let client_config = client_config(&config, &cache).unwrap();
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .nest("/api", api);
    let mut node_manager = NodeManager::default();
    for node in config
        .general
        .nodes
        .iter()
        .filter(|n| n.name != config.node.name)
    {
        let mut url = node.address.clone();
        let scheme = if url.scheme() == "http" { "ws" } else { "wss" };
        url.set_scheme(scheme).unwrap();

        let (ws, _) = tokio_tungstenite::connect_async(url.join("api/chatter").unwrap())
            .await
            .unwrap();
        let connection = connector::<_, ChatterMessage, ChatterMessage>(
            ws,
            ServerName::try_from(node.name.as_str()).unwrap(),
            client_config.clone(),
        )
        .await
        .unwrap();
        node_manager.up(node.name.clone(), Connection::connected(connection))
    }

    let server_config = server_config(&config, &cache).unwrap();

    // run it with hyper on localhost:3000
    axum::Server::bind(&config.node.addr)
        .serve(
            app.with_state(AppState {
                acceptor: Arc::new(Acceptor::from(server_config)),
                node_manager: Arc::new(RwLock::new(node_manager)),
                // client_config,
            })
            .into_make_service(),
        )
        .await
        .unwrap();
}
