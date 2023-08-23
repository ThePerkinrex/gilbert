use config::{GeneralConfig, GeneralConfigDiff};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum ChatterMessage {
    Hello {
        config: GeneralConfig,
        priority: u32,
        connected: Vec<String>,
    },
    QueueUpdate {
        length: u32,
    },
    NodeConfigUpdate {
        priority: u32,
    },
    GeneralConfigUpdate(GeneralConfigDiff),
    // SendTask {},
    // SendTaskResult {}
    Ping(u32),
    Pong(u32),
}
