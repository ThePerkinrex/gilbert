use std::{collections::HashSet, convert::Infallible, sync::Arc};

use async_trait::async_trait;
use config::Config;
use tokio::sync::RwLock;
use tokio_rustls::rustls::ClientConfig;

use super::NodeManager;

#[async_trait]
pub trait AttemptConnectHandler {
    type Error;
    async fn attempt_connect<'a, I: IntoIterator<Item = &'a str> + Send>(
        self: Arc<Self>,
        names: I,
    ) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait PongHandler {
    type Error;
    async fn pong(self: Arc<Self>, id: u32) -> Result<(), Self::Error>;
}

pub trait EventHandlers: AttemptConnectHandler + PongHandler {}
impl<T> EventHandlers for T where T: AttemptConnectHandler + PongHandler {}

pub trait FromErrors<Ev>:
    From<<Ev as PongHandler>::Error> + From<<Ev as AttemptConnectHandler>::Error>
where
    Ev: EventHandlers,
{
}

impl<T, Ev> FromErrors<Ev> for T
where
    Ev: EventHandlers,
    T: From<<Ev as PongHandler>::Error> + From<<Ev as AttemptConnectHandler>::Error>,
{
}

pub struct MockEv;

#[async_trait]
impl AttemptConnectHandler for MockEv {
    type Error = Infallible;

    async fn attempt_connect<'a, I: IntoIterator<Item = &'a str> + Send>(
        self: Arc<Self>,
        _: I,
    ) -> Result<(), Self::Error> {
        todo!()
    }
}

#[async_trait]
impl PongHandler for MockEv {
    type Error = Infallible;

    async fn pong(self: Arc<Self>, id: u32) -> Result<(), Self::Error> {
        println!("PONG: {id}");
        Ok(())
    }
}

#[derive(Clone)]
pub struct EventHandlersImpl {
    config: Arc<Config>,
    client_config: Arc<ClientConfig>,
    node_manager: Arc<RwLock<NodeManager>>,
}

impl EventHandlersImpl {
    pub fn new(
        config: Arc<Config>,
        client_config: Arc<ClientConfig>,
        node_manager: Arc<RwLock<NodeManager>>,
    ) -> Self {
        Self {
            config,
            client_config,
            node_manager,
        }
    }
}

#[async_trait]
impl AttemptConnectHandler for EventHandlersImpl {
    type Error = Infallible;

    async fn attempt_connect<'a, I: IntoIterator<Item = &'a str> + Send>(
        self: Arc<Self>,
        names: I,
    ) -> Result<(), Self::Error> {
        let names = names.into_iter().collect::<HashSet<_>>();
        let nodes = {
            let read = self.node_manager.read().await;
            self.config
                .general
                .nodes
                .iter()
                .filter(|n| names.contains(n.name.as_str()) && !read.get(&n.name).is_up())
                .collect::<Vec<_>>()
        };
        let mut write = self.node_manager.write().await;
        for node in nodes {
            write
                .connect(
                    node,
                    self.client_config.clone(),
                    self.config.clone(),
                    self.clone(),
                )
                .await
        }
        Ok(())
    }
}

#[async_trait]
impl PongHandler for EventHandlersImpl {
    type Error = Infallible;

    async fn pong(self: Arc<Self>, id: u32) -> Result<(), Self::Error> {
        println!("PONG: {id}");
        Ok(())
    }
}
