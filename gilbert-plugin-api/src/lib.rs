use std::{borrow::Cow, collections::HashMap};

use log::LogMessage;
use semver::Version;

pub mod log;
pub mod plugin_proto;
pub mod runner_proto;

pub const PROTO_VERSION: Version = Version::new(0, 1, 0);

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum GilbertRequest<Config, Proto> {
    Init {
        gilbert_version: Version,
        protocol_version: Version,
        config: Config,
    },
    IntoRunnerProtocol,
    Inner(Proto),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum GeneralPluginResponse<Proto> {
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
    Inner(Proto),
}

impl<Proto> From<LogMessage> for GeneralPluginResponse<Proto> {
    fn from(value: LogMessage) -> Self {
        Self::Log(value)
    }
}

impl<Proto> GeneralPluginResponse<Proto> {
    pub fn map<U, F: FnOnce(Proto) -> U>(self, mapping: F) -> GeneralPluginResponse<U> {
        match self {
            Self::Init {
                plugin_version,
                protocol_version_valid,
            } => GeneralPluginResponse::Init {
                plugin_version,
                protocol_version_valid,
            },
            Self::InitRunner {
                plugin_version,
                protocol_version_valid,
                accpeted_extensions,
            } => GeneralPluginResponse::InitRunner {
                plugin_version,
                protocol_version_valid,
                accpeted_extensions,
            },
            Self::Log(m) => GeneralPluginResponse::Log(m),
            Self::Inner(p) => GeneralPluginResponse::Inner(mapping(p)),
        }
    }
}
