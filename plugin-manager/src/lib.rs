use std::{path::{Path, PathBuf}, collections::HashMap, process::Stdio};

use futures_util::{SinkExt, StreamExt};
use gilbert_plugin_api::{GilbertRequest, GeneralPluginResponse, log::LogMessage};
use semver::Version;
use thiserror::Error;
use tokio_util::codec::{FramedWrite, LinesCodec, FramedRead, LinesCodecError};

#[derive(Debug, Error)]
pub enum PluginLoadError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Codec(#[from] LinesCodecError),
    #[error(transparent)]
    Serde(#[from] serde_json::Error)
}

pub async fn load<P: AsRef<Path> + Send, Config: serde::Serialize + Send + Sync>(path: P, gilbert_version: Version, config: Config) -> Result<(), PluginLoadError> {
    let init = GilbertRequest::<_, ()>::Init { gilbert_version, protocol_version: gilbert_plugin_api::PROTO_VERSION, config };
    let mut process = tokio::process::Command::new(path.as_ref())
        // .current_dir(todo!())
        .kill_on_drop(true)
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .stderr(Stdio::inherit()).spawn()?;
    let codec = LinesCodec::new();
    let mut framed_write = FramedWrite::new(process.stdin.take().unwrap(), codec.clone());
    let mut framed_read = FramedRead::new(process.stdout.take().unwrap(), codec);
    framed_write.send(serde_json::to_string(&init)?).await?;
    tokio::time::timeout(std::time::Duration::from_secs(5), tokio::spawn(async move {
        let mut framed_read = framed_read.map(|s| s.map_err(|e| PluginLoadError::Codec(e)).and_then(|s| serde_json::from_str::<GeneralPluginResponse<()>>(&s).map_err(PluginLoadError::Serde)));
        while let Some(Ok(data)) = framed_read.next().await {
            match data {
                GeneralPluginResponse::Init { plugin_version, protocol_version_valid } => todo!(),
                GeneralPluginResponse::InitRunner { plugin_version, protocol_version_valid, accpeted_extensions } => todo!(),
                GeneralPluginResponse::Log(msg) => match msg.level {
                    gilbert_plugin_api::log::Level::Trace => (),
                    gilbert_plugin_api::log::Level::Debug => println!("PLUGIN: {msg:?}"),
                    gilbert_plugin_api::log::Level::Info => println!("PLUGIN: {msg:?}"),
                    gilbert_plugin_api::log::Level::Warn => println!("PLUGIN: {msg:?}"),
                    gilbert_plugin_api::log::Level::Error => println!("PLUGIN: {msg:?}"),
                },
                GeneralPluginResponse::Inner(_) => todo!(),
            }
        }
    })).await.unwrap().unwrap();
    Ok(())
}

pub struct PluginManager {
    plugins_dir: PathBuf,
    
}