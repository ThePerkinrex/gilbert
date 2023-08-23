use std::sync::Arc;

use axum::{
    extract::{State, WebSocketUpgrade},
    response::Response,
    routing::get,
    Router,
};
use cache::{CacheError, CertificatesCache};
use config::Config;
use node_manager::{
    event_triggers::{AttemptConnectHandler, EventHandlers, EventHandlersImpl, PongHandler, FromErrors},
    Connection, ConnectionError, NodeManager,
};
use secure_comms::Acceptor;
use tokio::sync::RwLock;
use tokio_rustls::rustls::{
    server::AllowAnyAuthenticatedClient, ClientConfig, RootCertStore, ServerConfig,
};

use crate::node_manager::event_triggers::MockEv;

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
struct AppState<Ev> {
    acceptor: Arc<Acceptor>,
    node_manager: Arc<RwLock<NodeManager>>,
    config: Arc<Config>, // client_config: Arc<ClientConfig>,
    ev: Arc<Ev>,
}

async fn chatter<Ev>(State(state): State<AppState<Ev>>, ws: WebSocketUpgrade) -> Response
where
    Ev: EventHandlers + Send + Sync + 'static,
    ConnectionError: FromErrors<Ev>,
{
    ws.on_upgrade(|ws| async move {
        tokio::spawn(async move {
            let (s, name) = state.acceptor.accept_with_server_name(ws).await.unwrap();
            if let Some(name) = name {
                println!("Connected to {}", name);
                state.node_manager.write().await.up(
                    name,
                    Connection::accepted(s, state.config.clone(), state.ev.clone()),
                )
            } else {
                eprintln!("NO SNI PROVIDED");
            }
        });
    })
}

pub async fn start(config: Config) {
    let config = Arc::new(config);
    let api = Router::new().route("/chatter", get(chatter));
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
