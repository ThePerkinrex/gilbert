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
use node_manager::{
    event_triggers::{EventHandlers, EventHandlersImpl, FromErrors},
    Connection, ConnectionError, NodeManager,
};
use secure_comms::Acceptor;
use tokio::sync::RwLock;
use tokio_rustls::rustls::{
    server::AllowAnyAuthenticatedClient, ClientConfig, RootCertStore, ServerConfig,
};
use tracing::{error, info};

mod api;
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

struct AppState<Ev> {
    acceptor: Arc<Acceptor>,
    node_manager: Arc<RwLock<NodeManager>>,
    config: Arc<Config>, // client_config: Arc<ClientConfig>,
    ev: Arc<Ev>,
}

impl<Ev> Clone for AppState<Ev> {
    fn clone(&self) -> Self {
        Self {
            acceptor: self.acceptor.clone(),
            node_manager: self.node_manager.clone(),
            config: self.config.clone(),
            ev: self.ev.clone(),
        }
    }
}

pub async fn start(config: Config) {
    let config = Arc::new(config);
    let api = api::api();
    let cache = CertificatesCache::default();
    let client_config = client_config(&config, &cache).unwrap();
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .nest("/api", api);
    let node_manager = NodeManager::default();
    let node_manager = Arc::new(RwLock::new(node_manager));
    let ev = Arc::new(EventHandlersImpl::new(
        config.clone(),
        client_config.clone(),
        node_manager.clone(),
    ));
    for node in config
        .general
        .nodes
        .iter()
        .filter(|n| n.name != config.node.name)
    {
        node_manager
            .write()
            .await
            .connect(node, client_config.clone(), config.clone(), ev.clone())
            .await;
    }

    let server_config = server_config(&config, &cache).unwrap();

    // run it with hyper on localhost:3000
    info!("Server started at {}", config.node.addr);
    axum::Server::bind(&config.node.addr)
        .serve(
            app.with_state(AppState {
                acceptor: Arc::new(Acceptor::from(server_config)),
                node_manager,
                config, // client_config,
                ev,
            })
            .into_make_service(),
        )
        .await
        .unwrap();
}
