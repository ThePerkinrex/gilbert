use std::task::Poll;

use axum::extract::ws::{Message, WebSocket};
use futures_util::{Sink, Stream};
use tokio::io::{AsyncRead, AsyncWrite};

use crate::WebSocketByteStream;

impl AsyncRead for WebSocketByteStream<WebSocket> {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let projection = self.project();
        let res = projection.socket.poll_next(cx);
        match res {
            Poll::Ready(data) => match data {
                None | Some(Ok(Message::Close(_))) => Poll::Ready(Err(std::io::Error::new(
                    std::io::ErrorKind::ConnectionReset,
                    "Web socket closed",
                ))),
                Some(Err(e)) => Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::Other, e))),
                Some(Ok(Message::Binary(b))) => {
                    buf.put_slice(&b);
                    Poll::Ready(Ok(()))
                }
                Some(Ok(Message::Text(_))) => Poll::Ready(Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "WebSocket received text kind data",
                ))),
                Some(Ok(Message::Ping(_))) => Poll::Ready(Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "WebSocket received ping",
                ))),
                Some(Ok(Message::Pong(_))) => Poll::Ready(Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "WebSocket received pong",
                ))),
            },
            Poll::Pending => Poll::Pending,
        }
    }
}

impl AsyncWrite for WebSocketByteStream<WebSocket> {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        let mut project = self.project();
        let mut socket = project.socket.as_mut();

        match socket.as_mut().poll_ready(cx) {
            Poll::Ready(Ok(())) => (),
            Poll::Ready(Err(e)) => {
                return Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::Other, e)))
            }
            Poll::Pending => return Poll::Pending,
        }

        if let Err(e) = socket.as_mut().start_send(Message::Binary(buf.to_vec())) {
            return Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::Other, e)));
        }
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        self.project()
            .socket
            .poll_flush(cx)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        self.project()
            .socket
            .poll_close(cx)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}
