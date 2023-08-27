use std::collections::HashMap;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct LogMessage {
    pub level: Level,
    pub name: String,
    pub target: String,
    pub module_path: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub fields: HashMap<String, serde_json::Value>,
}
