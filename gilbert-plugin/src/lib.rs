use std::{process::exit, convert::Infallible};
use std::future::Future;

use gilbert_plugin_api::AlfredRequest;
// use alfred_plugin_api::{log::LogMessage, PluginResponse};
use futures_util::{Sink, Stream, StreamExt};
use semver::Version;
use serde::Deserialize;
use subscriber::LoggingLayer;
use thiserror::Error;
use tokio::io::{stdin, stdout};
use tokio::select;
use tokio::sync::Notify;
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec, LinesCodecError};
use tracing::{error, info};
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

mod subscriber;

pub trait Plugin {}

#[derive(Debug, Error)]
enum RunError {
    #[error(transparent)]
    Codec(#[from] LinesCodecError),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error("Unable to parse init message: {0}")]
    UnableToParseInit(#[source] serde_json::Error)
}

async fn init_plugin_fn_internal<'a, Config, Init, P, E1, E2, FR, FW, Exit, FutExit>(
    version: Version,
    init: Init,
    read: &mut FR,
    write: &mut FW,
    exit: Exit,
    exit_shot: tokio::sync::oneshot::Receiver<Notify>
) -> Result<(), RunError>
where
    Config: Deserialize<'a>,
    Init: FnOnce(Config) -> P + Send,
    P: Plugin,
    RunError: From<E1> + From<E2>,
    FR: Stream<Item = Result<String, E1>> + Unpin + Send,
    FW: Sink<String, Error = E2> + Send,
    E1: Send,
    E2: Send,
    Exit: FnOnce(i32) -> FutExit,
    FutExit: Future<Output = Infallible>
{
    let Some(init) = read.next().await else {
        return Ok(());
    };
    let init = init?;
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    tracing_subscriber::registry().with(LoggingLayer::new(tx)).init();

    tokio::spawn(async move {
        while let Some(s) = rx.recv().await {
            println!("{}", serde_json::to_string(&s).unwrap());
        }
    });
    let init = serde_json::from_str::<AlfredRequest>(&init).map_err(RunError::UnableToParseInit)?;
    let AlfredRequest::Init { gilbert_version: alfred_version, protocol_version, config } = init else {
        error!("init message isn't first message");
        exit(2).await;
        unreachable!()
    };
    Ok(())
}

pub async fn init_plugin_fn<'a, Config, Init, P>(version: Version, init: Init) -> !
where
    Config: Deserialize<'a>,
    Init: FnOnce(Config) -> P + Send,
    P: Plugin,
{
    let codec = LinesCodec::new();
    let mut frame_read = FramedRead::new(stdin(), codec.clone());
    let mut frame_write = FramedWrite::new(stdout(), codec);
    match init_plugin_fn_internal(version, init, &mut frame_read, &mut frame_write).await {
        Ok(()) => exit(0),
        Err(e) => {
            error!("Plugin error: {e}");
            exit(1)
        }
    }
}
