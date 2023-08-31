use std::{borrow::Cow, collections::HashMap, convert::Infallible, sync::Arc};

use chatter_protocol::ChatterMessage;
use config::{Config, Node};
use futures_util::{stream::SplitSink, Sink, SinkExt, Stream, StreamExt};
use pin_project::pin_project;
use secure_comms::{connector, DataStream, DataStreamError, WebSocketByteStream};
use thiserror::Error;
use tokio::{net::TcpStream, sync::RwLock, task::JoinHandle};
use tokio_rustls::rustls::ServerName;
use tokio_tungstenite::MaybeTlsStream;
use tracing::{debug, error, info};

use self::event_triggers::{EventHandlers, FromErrors};

pub mod event_triggers;

#[derive(Default)]
pub struct NodeManager {
    nodes: HashMap<Arc<str>, NodeStatus>,
}

impl NodeManager {
    pub fn get(&self, key: &str) -> Cow<NodeStatus> {
        self.nodes
            .get(key)
            .map_or(Cow::Owned(NodeStatus::Unknown), Cow::Borrowed)
    }

    pub fn down<S: Into<Arc<str>>>(&mut self, key: S) {
        self.nodes.insert(key.into(), NodeStatus::Down);
    }

    pub fn up<S: Into<Arc<str>>>(&mut self, key: S, connection: Connection) {
        self.nodes.insert(key.into(), NodeStatus::Up(connection));
    }

    pub fn connected(&self) -> impl Iterator<Item = Arc<str>> + '_ {
        self.nodes
            .iter()
            .filter(|(_, x)| x.is_up())
            .map(|(x, _)| x.clone())
    }

    pub fn nodes(&self) -> impl Iterator<Item = (&str, &NodeStatus)> + '_ {
        self.nodes
            .iter()
            .map(|(x, y)| (&**x, y))
    }

    pub async fn connect<Ev>(
        &mut self,
        node: &Node,
        client_config: Arc<tokio_rustls::rustls::ClientConfig>,
        config: Arc<Config>,
        ev: Arc<Ev>,
    ) where
        Ev: EventHandlers + Send + Sync + 'static,
        ConnectionError: FromErrors<Ev>,
    {
        let mut url = node.address.clone();
        let scheme = if url.scheme() == "http" { "ws" } else { "wss" };
        url.set_scheme(scheme).unwrap();

        match tokio_tungstenite::connect_async(url.join("api/chatter").unwrap()).await {
            Ok((ws, _)) => {
                let connection = connector::<_, ChatterMessage, ChatterMessage>(
                    ws,
                    ServerName::try_from(node.name.as_str()).unwrap(),
                    client_config.clone(),
                )
                .await
                .unwrap();
                info!("Connected to {} @ {}", node.name, node.address);
                let connection =
                    Connection::connected(connection, config.clone(), ev, node.name.clone());
                let msg = ChatterMessage::Hello {
                    config: config.general.clone(),
                    priority: config.node.priority,
                    connected: self.connected().map(|s| s.to_string()).collect(),
                };
                connection.send(msg).await.unwrap();
                self.up(node.name.clone(), connection)
            }
            Err(e) => {
                error!(
                    "Error connecting to {} @ {}: {}",
                    node.name, node.address, e
                );
                self.down(node.name.clone())
            }
        }
    }
}

#[derive(Default, Clone)]
pub enum NodeStatus {
    Down,
    #[default]
    Unknown,
    Up(Connection),
}

impl NodeStatus {
    const fn is_up(&self) -> bool {
        matches!(self, Self::Up(_))
    }
}

type DataStreamShortServer<S, I, O = I> =
    DataStream<tokio_rustls::server::TlsStream<WebSocketByteStream<S>>, I, O>;
type DataStreamShortClient<S, I, O = I> =
    DataStream<tokio_rustls::client::TlsStream<WebSocketByteStream<S>>, I, O>;
type SplitSinkDataStreamServer<S, M> = SplitSink<DataStreamShortServer<S, M>, M>;
type SplitSinkDataStreamClient<S, M> = SplitSink<DataStreamShortClient<S, M>, M>;

pub struct Connection<M = ChatterMessage> {
    sink: Arc<RwLock<ConnectionSink<M>>>,
    handle: Arc<JoinHandle<Result<(), ConnectionError>>>,
    state: Arc<RwLock<ConnState>>,
}

struct ConnState {
    priority: u32,
}

impl<M> Clone for Connection<M> {
    fn clone(&self) -> Self {
        Self {
            sink: self.sink.clone(),
            handle: self.handle.clone(),
            state: self.state.clone(),
        }
    }
}

#[pin_project(project = ConnectionProj)]
pub enum ConnectionSink<M> {
    Accepted {
        #[pin]
        sink: SplitSinkDataStreamServer<axum::extract::ws::WebSocket, M>,
    },
    Connected {
        #[pin]
        sink: SplitSinkDataStreamClient<
            tokio_tungstenite::WebSocketStream<MaybeTlsStream<TcpStream>>,
            M,
        >,
    },
}

#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("Configs dont match")]
    ConfigsDontMatch,
    #[error(transparent)]
    DataStream(#[from] DataStreamError),
}

impl From<Infallible> for ConnectionError {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

impl Connection<ChatterMessage> {
    pub fn accepted<Ev, Name>(
        stream: DataStreamShortServer<axum::extract::ws::WebSocket, ChatterMessage>,
        config: Arc<Config>,
        ev: Arc<Ev>,
        name: Name,
    ) -> Self
    where
        Ev: EventHandlers + Send + Sync + 'static,
        ConnectionError: FromErrors<Ev>,
        Name: AsRef<str> + Send + Sync + 'static,
    {
        let (sink, stream) = stream.split();
        let sink = Arc::new(RwLock::new(ConnectionSink::Accepted { sink }));
        let state = Arc::new(RwLock::new(ConnState { priority: 0 }));
        let handle = tokio::spawn(Self::receiver(
            stream,
            sink.clone(),
            config,
            ev,
            state.clone(),
            name,
        ));
        Self {
            sink,
            handle: Arc::new(handle),
            state,
        }
    }

    pub fn connected<Ev, Name>(
        stream: DataStreamShortClient<
            tokio_tungstenite::WebSocketStream<MaybeTlsStream<TcpStream>>,
            ChatterMessage,
        >,
        config: Arc<Config>,
        ev: Arc<Ev>,
        name: Name,
    ) -> Self
    where
        Ev: EventHandlers + Send + Sync + 'static,
        ConnectionError: FromErrors<Ev>,
        Name: AsRef<str> + Send + Sync + 'static,
    {
        let (sink, stream) = stream.split();
        let sink = Arc::new(RwLock::new(ConnectionSink::Connected { sink }));
        let state = Arc::new(RwLock::new(ConnState { priority: 0 }));
        let handle = tokio::spawn(Self::receiver(
            stream,
            sink.clone(),
            config,
            ev,
            state.clone(),
            name,
        ));
        Self {
            sink,
            handle: Arc::new(handle),
            state,
        }
    }

    async fn receiver<
        St: Stream<Item = Result<ChatterMessage, DataStreamError>> + Send + Unpin,
        Si,
        Ev,
        Name,
    >(
        mut stream: St,
        sink: Arc<RwLock<Si>>,
        config: Arc<Config>,
        ev: Arc<Ev>,
        state: Arc<RwLock<ConnState>>,
        name: Name,
    ) -> Result<(), ConnectionError>
    where
        Si: Sink<ChatterMessage, Error = DataStreamError> + Unpin + Sync + Send,
        Ev: EventHandlers + Send + Sync,
        ConnectionError: FromErrors<Ev>,
        Name: AsRef<str> + Send + Sync,
    {
        let res = loop {
            match stream.next().await {
                None => break Ok(()),
                Some(Ok(msg)) => match msg {
                    ChatterMessage::QueueUpdate { length: _ } => todo!(),
                    ChatterMessage::NodeConfigUpdate { priority: _ } => todo!(),
                    ChatterMessage::GeneralConfigUpdate(_) => todo!(),
                    ChatterMessage::Ping(x) => {
                        let _ = sink.write().await.send(ChatterMessage::Pong(x)).await;
                    }
                    ChatterMessage::Pong(x) => {
                        ev.clone().pong(x).await?;
                    }
                    ChatterMessage::Hello {
                        config: c,
                        priority,
                        connected,
                    } => {
                        debug!(name = name.as_ref(), "Hello");
                        // dbg!(&c);
                        // dbg!(&config.general);
                        // dbg!(c == config.general);
                        if c != config.general {
                            break Err(ConnectionError::ConfigsDontMatch);
                        }
                        state.write().await.priority = priority;
                        ev.clone()
                            .attempt_connect(connected.iter().map(String::as_str))
                            .await?;
                    }
                },
                Some(Err(e)) => break Err(e.into()),
            }
        };
        if let Err(e) = &res {
            error!(name = name.as_ref(), "Connection error: {e}");
        }
        res
    }

    pub async fn send(&self, msg: ChatterMessage) -> Result<(), DataStreamError> {
        self.sink.write().await.send(msg).await
    }
}

impl<M> Sink<M> for ConnectionSink<M>
where
    M: serde::Serialize,
{
    type Error = DataStreamError;

    fn poll_ready(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        match self.project() {
            ConnectionProj::Accepted { sink, .. } => sink.poll_ready(cx),
            ConnectionProj::Connected { sink, .. } => sink.poll_ready(cx),
        }
    }

    fn start_send(self: std::pin::Pin<&mut Self>, item: M) -> Result<(), Self::Error> {
        match self.project() {
            ConnectionProj::Accepted { sink, .. } => sink.start_send(item),
            ConnectionProj::Connected { sink, .. } => sink.start_send(item),
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        match self.project() {
            ConnectionProj::Accepted { sink, .. } => sink.poll_flush(cx),
            ConnectionProj::Connected { sink, .. } => sink.poll_flush(cx),
        }
    }

    fn poll_close(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        match self.project() {
            ConnectionProj::Accepted { sink, .. } => sink.poll_close(cx),
            ConnectionProj::Connected { sink, .. } => sink.poll_close(cx),
        }
    }
}
