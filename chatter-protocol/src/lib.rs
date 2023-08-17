use config::ConfigMods as GeneralConfig;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum ChatterMessage {
    QueueUpdate { length: u32 },
    NodeConfigUpdate { priority: u32 },
    GeneralConfigUpdate(GeneralConfig),
    // SendTask {},
    // SendTaskResult {}
}
