use config::GeneralConfigDiff as GeneralConfig;

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub enum ChatterMessage {
    QueueUpdate { length: u32 },
    NodeConfigUpdate { priority: u32 },
    GeneralConfigUpdate(GeneralConfig),
    // SendTask {},
    // SendTaskResult {}
    Ping(u32),
    Pong(u32),
}
