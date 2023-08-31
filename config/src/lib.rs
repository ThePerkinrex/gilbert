use diff::Diff;
use url::Url;
use std::{collections::HashMap, net::SocketAddr, path::PathBuf};
use url_diff::DiffUrl;

#[cfg(feature = "schemars")]
use schemars::JsonSchema;

mod url_diff;
pub mod repo;

#[derive(Debug, serde::Deserialize, serde::Serialize, Diff, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[diff(attr(
    #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
))]
pub struct GeneralConfig {
    pub nodes: Vec<Node>,
    pub tasks: HashMap<String, TaskInfo>,
    #[serde(default)]
    pub plugins: Vec<Plugin>
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Diff, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[diff(attr(
    #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
))]
#[serde(untagged)]
pub enum Plugin {
    Name(String),
    NameAndVersion {
        name: String,
        version: String
    },
    NameWithRepo {
        name: String,
        repo: String,
        #[serde(default)]
        version: Option<String>
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct Config {
    pub general: GeneralConfig,
    pub node: NodeConfig,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct NodeConfig {
    pub ca_file: PathBuf,
    pub cert_file: PathBuf,
    pub key_file: PathBuf,
    pub addr: SocketAddr,
    pub name: String,
    #[serde(default)]
    pub priority: u32,
    // pub repos: HashMap<String, Source>
}

#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(tag = "source")]
pub enum Source {
    #[serde(rename = "fs", alias = "filesystem")]
    Fs {
        path: PathBuf
    },
    #[serde(rename = "git")]
    Git {
        repo: String,
        #[serde(default)]
        version: Option<String>,
        #[serde(default)]
        path: Option<PathBuf>
    },
    #[serde(rename = "web")]
    Web {
        url: Url
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Diff, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[diff(attr(
    #[derive(Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize, Clone)]
))]
pub struct Node {
    pub address: DiffUrl,
    pub name: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Diff, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[diff(attr(
    #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
))]
pub struct TaskInfo {
    pub params: Vec<Param>,
    #[serde(default)]
    pub allowed_nodes: Option<Vec<String>>,
    #[serde(default)]
    pub disallowed_nodes: Option<Vec<String>>,
    pub script: PathBuf,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Diff, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[diff(attr(
    #[derive(Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize, Clone)]
))]
pub struct Param {
    name: String,
    #[serde(rename = "type")]
    ty: ParamType,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Diff, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[diff(attr(
    #[derive(Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize, Clone)]
))]
#[serde(rename_all = "lowercase")]
pub enum ParamType {
    Number,
    String,
    Object,
    Array,
}

// #[derive(Debug, serde::Deserialize, serde::Serialize)]
// pub enum ScriptSource {
//     Mem(String),
// }

// #[derive(Debug, serde::Deserialize, serde::Serialize)]
// pub struct ConfigMods {
//     pub nodes: Option<Vec<NodeMods>>,
//     pub tasks: Option<HashMap<String, TaskInfoMods>>,
// }

// #[derive(Debug, serde::Deserialize, serde::Serialize)]
// pub struct NodeMods {
//     pub address: Option<String>,
//     pub name: Option<String>,
// }

// #[derive(Debug, serde::Deserialize, serde::Serialize)]
// pub struct TaskInfoMods {
//     pub params: Option<Vec<Param>>,
//     pub script: Option<PathBuf>
// }
