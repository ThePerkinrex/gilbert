use std::sync::Arc;

use pin_project::pin_project;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_rustls::{rustls::{ServerConfig, ServerName, ClientConfig}, server, client, TlsAcceptor, TlsConnector};

#[pin_project]
#[derive(Debug)]
pub struct WebSocketByteStream<W> {
    #[pin]
    socket: W,
}

pub async fn acceptor<W: Send>(
    ws: W,
    config: Arc<ServerConfig>,
) -> std::io::Result<server::TlsStream<WebSocketByteStream<W>>>
where
    WebSocketByteStream<W>: AsyncRead + AsyncWrite + Unpin,
{
    TlsAcceptor::from(config)
        .accept(WebSocketByteStream { socket: ws })
        .await
}

pub async fn connector<W: Send>(
    ws: W,
    domain: ServerName,
    config: Arc<ClientConfig>,
) -> std::io::Result<client::TlsStream<WebSocketByteStream<W>>>
where
    WebSocketByteStream<W>: AsyncRead + AsyncWrite + Unpin,
{
    TlsConnector::from(config)
        .connect(domain, WebSocketByteStream { socket: ws })
        .await
}

pub mod axum_ws;
pub mod tungstenite;
