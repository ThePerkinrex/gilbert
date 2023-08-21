use std::path::PathBuf;

use deno_core::ModuleSpecifier;
use runner::{RunParams, SimplePrinter};
use serde_json::json;

#[tokio::main]
async fn main() {
    runner::run(RunParams {
        main_module: ModuleSpecifier::from_file_path(
            PathBuf::from("./runner/exampls/example.job.js")
                .canonicalize()
                .unwrap(),
        )
        .unwrap(),
        printer: SimplePrinter,
        params: vec![json!(1), json!(2)],
    })
    .await
    .unwrap()
}
