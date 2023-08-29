use std::{fs::File, path::PathBuf};

use config::{Config, repo::Repository};
use schemars::schema_for;

fn main() {
    let config = schema_for!(Config);
    let repo = schema_for!(Repository);
    let path = PathBuf::from(std::env::args().nth(1).unwrap());
    let config_file = File::create(path.join("config.schema.json")).unwrap();
    let repo_file = File::create(path.join("repo.schema.json")).unwrap();
    serde_json::to_writer_pretty(config_file, &config).unwrap();
    serde_json::to_writer_pretty(repo_file, &repo).unwrap();
}
