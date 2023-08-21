use std::{collections::HashMap, path::PathBuf};
use diff::Diff;
use url_diff::DiffUrl;
mod url_diff;

#[derive(Debug, serde::Deserialize, serde::Serialize, Diff, PartialEq, Eq)]
#[diff(attr(
    #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
))]
pub struct Config {
    pub nodes: Vec<Node>,
    pub tasks: HashMap<String, TaskInfo>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Diff, PartialEq, Eq)]
#[diff(attr(
    #[derive(Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
))]
pub struct Node {
    pub address: DiffUrl,
    pub name: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Diff, PartialEq, Eq)]
#[diff(attr(
    #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
))]
pub struct TaskInfo {
    pub params: Vec<Param>,
    #[serde(default)]
    pub allowed_nodes: Option<Vec<String>>,
    #[serde(default)]
    pub disallowed_nodes: Option<Vec<String>>,
    pub script: PathBuf
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Diff, PartialEq, Eq)]
#[diff(attr(
    #[derive(Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
))]
pub struct Param {
    name: String,
    #[serde(rename = "type")]
    ty: ParamType
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Diff, PartialEq, Eq)]
#[diff(attr(
    #[derive(Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
))]
#[serde(rename_all = "lowercase")] 
pub enum ParamType {
    Number,
    String,
    Object,
    Array
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
