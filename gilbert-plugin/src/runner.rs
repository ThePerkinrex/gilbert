use futures_util::{Stream, StreamExt};
use gilbert_plugin_api::{PluginResponse, GilbertRequest, runner_proto::RunnerResponse};
use semver::{Version, VersionReq};
use serde::Deserialize;
use tracing::info;

use crate::{RunError, sender::Sender};

pub trait Runner {}

pub(crate) async fn init_runner_fn_internal<Config, Init, P, E1, FR, S>(
    version: Version,
    init: Init,
    read: &mut FR,
    sender: S,
) -> Result<(), RunError>
where
    Config: for<'a> Deserialize<'a>,
    Init: FnOnce(Config) -> P + Send,
    P: Runner,
    RunError: From<E1>,
    FR: Stream<Item = Result<String, E1>> + Unpin + Send,
    E1: Send,
    S: Sender<serde_json::Value> + Send,
{
    let Some(init_msg) = read.next().await else {
        return Ok(());
    };
    let init_msg = init_msg?;

    let init_msg = serde_json::from_str::<GilbertRequest<Config>>(&init_msg)
        .map_err(RunError::UnableToParseInit)?;
    let GilbertRequest::Init {
        gilbert_version,
        protocol_version,
        config,
    } = init_msg
    else {
        return Err(RunError::InitIsntFirstMessage);
    };
    let valid = VersionReq::parse(&format!("^{}", gilbert_plugin_api::PROTO_VERSION))
        .unwrap()
        .matches(&protocol_version);
    sender.map_temp(|p| serde_json::to_value(&p).unwrap()).send(PluginResponse::InitRunner {
        plugin_version: version,
        protocol_version_valid: valid,
		accpeted_extensions: vec![]
    });
    if !valid {
        return Err(RunError::VersionIncompatible {
            implemented: gilbert_plugin_api::PROTO_VERSION,
            host: protocol_version,
        });
    }
    info!("Connected to gilbert v{gilbert_version}");
    // TODO
    Ok(())
}