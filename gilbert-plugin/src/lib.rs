use std::error::Error;

use futures_util::{Sink, SinkExt};
use gilbert_plugin_api::log::LogMessage;
use plugin::init_plugin_fn_internal;
use runner::init_runner_fn_internal;
use semver::Version;
use serde::Deserialize;
use subscriber::LoggingLayer;
use thiserror::Error;
use tokio::io::{stdin, stdout};
use tokio::select;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec, LinesCodecError};
use tracing::error;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

pub mod plugin;
pub mod runner;
pub mod sender;
mod subscriber;

pub use plugin::PluginBuilder;
pub use runner::RunnerBuilder;

#[derive(Debug, Error)]
enum RunError {
    #[error(transparent)]
    Codec(#[from] LinesCodecError),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error("Unable to parse init message: {0}")]
    UnableToParseInit(#[source] serde_json::Error),
    #[error("init isnt first message")]
    InitIsntFirstMessage,
    #[error(
        "Protocol version incompatible: host has {host} and our current build is {implemented}"
    )]
    VersionIncompatible { implemented: Version, host: Version },
    #[error(transparent)]
    SpecificError(Box<dyn Error + Send>)
}

enum PrinterState {
    Normal,
    Finishing,
    Finished,
}

async fn print_message<T: serde::Serialize + Sync, FW: Sink<String, Error = E> + Send + Unpin, E>(
    sink: &mut FW,
    msg: &T,
) -> Result<(), RunError>
where
    RunError: From<E>,
{
    sink.send(serde_json::to_string(msg)?)
        .await
        .map_err(Into::into)
}

struct Exit {
    tx_state: UnboundedSender<PrinterState>,
    handle: JoinHandle<Result<(), RunError>>,
}

impl Exit {
    pub async fn exit(self, code: i32) -> ! {
        let _ = self.tx_state.send(PrinterState::Finishing);
        let _ = self.handle.await;
        std::process::exit(code);
    }
}

async fn load_plugin<T>() -> (
    tokio::sync::mpsc::UnboundedSender<T>,
    Exit,
    FramedRead<tokio::io::Stdin, LinesCodec>,
)
where
    T: serde::Serialize + From<LogMessage> + Send + Sync + 'static,
{
    let codec = LinesCodec::new();
    let frame_read = FramedRead::new(stdin(), codec.clone());
    let mut frame_write = FramedWrite::new(stdout(), codec);
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let (tx_state, mut rx_state) = tokio::sync::mpsc::unbounded_channel();
    tracing_subscriber::registry()
        .with(LoggingLayer::new(tx.clone()))
        .init();

    let s = tokio::spawn(async move {
        let mut state = PrinterState::Normal;
        loop {
            #[allow(clippy::redundant_pub_crate)]
            match state {
                PrinterState::Normal => select! {
                    Some(resp) = rx.recv() => print_message(&mut frame_write, &resp).await?,
                    Some(new_state) = rx_state.recv() => {state = new_state;}
                    else => break
                },
                PrinterState::Finishing => match rx.try_recv() {
                    Ok(resp) => print_message(&mut frame_write, &resp).await?,
                    Err(_) => state = PrinterState::Finished,
                },
                PrinterState::Finished => break,
            };
        }
        Ok::<(), RunError>(())
    });

    (
        tx,
        Exit {
            tx_state,
            handle: s,
        },
        frame_read,
    )
}

pub async fn init_plugin_fn<Config, Init, P>(version: Version, init: Init) -> !
where
    Config: for<'a> Deserialize<'a>,
    Init: FnOnce(Config) -> P + Send,
    P: PluginBuilder,
{
    let (tx, exit, mut frame_read) = load_plugin().await;
    match init_plugin_fn_internal(version, init, &mut frame_read, tx).await {
        Ok(()) => exit.exit(0).await,
        Err(e) => {
            error!("{e}");
            exit.exit(1).await
        }
    }
}

pub async fn init_plugin<Config, P>(version: Version) -> !
where
    Config: for<'a> Deserialize<'a>,
    P: PluginBuilder + From<Config>,
{
    init_plugin_fn::<Config, _, P>(version, Into::into).await
}

pub async fn init_runner_fn<Config, Init, P>(version: Version, init: Init) -> !
where
    Config: for<'a> Deserialize<'a>,
    Init: FnOnce(Config) -> P + Send,
    P: RunnerBuilder,
{
    let (tx, exit, mut frame_read) = load_plugin().await;
    match init_runner_fn_internal(version, init, &mut frame_read, tx).await {
        Ok(()) => exit.exit(0).await,
        Err(e) => {
            error!("{e}");
            exit.exit(1).await
        }
    }
}

pub async fn init_runner<Config, P>(version: Version) -> !
where
    Config: for<'a> Deserialize<'a>,
    P: RunnerBuilder + From<Config>,
{
    init_runner_fn::<Config, _, P>(version, Into::into).await
}
