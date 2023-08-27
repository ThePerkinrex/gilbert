use std::{borrow::Cow, collections::HashMap};

use log::LogMessage;
use semver::Version;

pub mod log;
pub mod runner_proto;

pub const PROTO_VERSION: Version = Version::new(0, 1, 0);

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum GilbertRequest<Config> {
    Init {
        gilbert_version: Version,
        protocol_version: Version,
        config: Config,
    },
    IntoRunnerProtocol,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum PluginResponse {
    Init {
        plugin_version: Version,
        protocol_version_valid: bool, // TODO Handlers
    },
    InitRunner {
        plugin_version: Version,
        protocol_version_valid: bool,
        #[serde(default)]
        accpeted_extensions: Vec<Cow<'static, str>>,
    },
    Log(LogMessage),
}

impl From<LogMessage> for PluginResponse {
    fn from(value: LogMessage) -> Self {
        Self::Log(value)
    }
}
