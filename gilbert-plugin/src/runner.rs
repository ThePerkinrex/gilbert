use std::borrow::Cow;

use futures_util::{Stream, StreamExt};
use gilbert_plugin_api::{runner_proto::RunnerResponse, GeneralPluginResponse, GilbertRequest};
use semver::{Version, VersionReq};
use serde::Deserialize;
use tracing::info;

use crate::{sender::Sender, RunError};

pub trait RunnerBuilder {
    fn accepted_extensions(&self) -> Vec<Cow<'static, str>>;

    type Built<S: Sender<RunnerResponse>>: Runner;

    fn build<S: Sender<RunnerResponse>>(self, sender: S) -> Self::Built<S>;
}

pub trait Runner {

}

pub(crate) async fn init_runner_fn_internal<Config, Init, P, E1, FR, S>(
    version: Version,
    init: Init,
    read: &mut FR,
    sender: S,
) -> Result<(), RunError>
where
    Config: for<'a> Deserialize<'a>,
    Init: FnOnce(Config) -> P + Send,
    P: RunnerBuilder,
    RunError: From<E1>,
    FR: Stream<Item = Result<String, E1>> + Unpin + Send,
    E1: Send,
    S: Sender<GeneralPluginResponse<serde_json::Value>> + Send,
{
    let Some(init_msg) = read.next().await else {
        return Ok(());
    };
    let init_msg = init_msg?;

    let init_msg = serde_json::from_str::<GilbertRequest<Config, ()>>(&init_msg)
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
    let runner_builder = init(config);
    sender.send(GeneralPluginResponse::InitRunner {
        plugin_version: version,
        protocol_version_valid: valid,
        accpeted_extensions: runner_builder.accepted_extensions(),
    });
    if !valid {
        return Err(RunError::VersionIncompatible {
            implemented: gilbert_plugin_api::PROTO_VERSION,
            host: protocol_version,
        });
    }
    info!("Connected to gilbert v{gilbert_version}");
    let sender = sender.map_temp(|g: RunnerResponse| {
        GeneralPluginResponse::Inner(serde_json::to_value(g).unwrap())
    });
    let runner = runner_builder.build(sender);
    // TODO
    Ok(())
}
