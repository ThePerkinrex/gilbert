use std::fs::File;

use config::Config;
use schemars::schema_for;

fn main() {
    let schema = schema_for!(Config);
    let file = File::create(std::env::args().nth(1).unwrap()).unwrap();
    serde_json::to_writer_pretty(file, &schema).unwrap();
}
