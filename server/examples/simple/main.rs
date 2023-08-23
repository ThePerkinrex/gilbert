use std::fs::File;

#[tokio::main]
async fn main() {
    let config =
        serde_json::from_reader(File::open(std::env::args().nth(1).unwrap()).unwrap()).unwrap();
    server::start(config).await
}
