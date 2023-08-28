use std::error::Error;

pub enum StageResult {
	FinishedStage,
	FinishedJob(serde_json::Value)
}

#[async_trait::async_trait]
pub trait Job {
	type Err: Error;
	fn stages(&self) -> Vec<String>;
	async fn run_stage(&self, stage: &str) -> Result<StageResult, Self::Err>;
}