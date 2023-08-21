use config::GeneralConfig;

fn main() {
    dbg!(serde_json::from_str::<GeneralConfig>(include_str!("config.example.json")).unwrap());
}
