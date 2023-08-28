use std::{borrow::Cow, error::Error};

use futures_util::{Stream, StreamExt};
use gilbert_plugin_api::{runner_proto::RunnerResponse, GeneralPluginResponse, GilbertRequest};
use semver::{Version, VersionReq};
use serde::Deserialize;
use thiserror::Error;
use tracing::info;

use crate::{sender::Sender, RunError};

use self::job::Job;
pub mod job;

pub trait RunnerBuilder {
    fn accepted_extensions(&self) -> Vec<Cow<'static, str>>;

    type Built<S: Sender<RunnerResponse>>: Runner;

    fn build<S: Sender<RunnerResponse>>(self, sender: S) -> Self::Built<S>;
}

#[async_trait::async_trait]
pub trait Runner {
    type Job<'a>: Job where Self: 'a;
    type Err: Error;

    async fn start_job(&self, params: Vec<serde_json::Value>) -> Result<Self::Job<'_>, Self::Err>;
}

#[derive(Debug, Error)]
pub enum RunnerError<RunnerErr: Error, JobErr: Error> {
    #[error("Error starting job: {0}")]
    JobStart(RunnerErr),
    #[error("Error starting stage: {0}")]
    StageRun(JobErr)
}

impl<RunnerErr: Error + 'static, JobErr: Error + 'static> From<RunnerError<RunnerErr, JobErr>> for RunError where RunnerError<RunnerErr, JobErr>: Send {
    fn from(val: RunnerError<RunnerErr, JobErr>) -> Self {
        Self::SpecificError(Box::new(val))
    }
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
    // let r: Result<(), RunnerError<<<P as RunnerBuilder>::Built<_> as Runner>::Err, <<<P as RunnerBuilder>::Built<_> as Runner>::Job<'_> as Job>::Err>> = async move {
        
    //     Ok(())
    // };
    // TODO
    Ok(())
}
