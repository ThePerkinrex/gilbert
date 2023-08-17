use std::collections::HashMap;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Config {
    pub nodes: Vec<Node>,
    pub tasks: HashMap<String, TaskInfo>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Node {
    pub address: String,
    pub name: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct TaskInfo {
    pub params: Vec<String>,
    // pub script: ScriptSource
}

// #[derive(Debug, serde::Deserialize, serde::Serialize)]
// pub enum ScriptSource {
//     Mem(String),
// }

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ConfigMods {
    pub nodes: Option<Vec<NodeMods>>,
    pub tasks: Option<HashMap<String, TaskInfoMods>>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct NodeMods {
    pub address: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct TaskInfoMods {
    pub params: Option<Vec<String>>,
    // pub script: Option<ScriptSource>
}
