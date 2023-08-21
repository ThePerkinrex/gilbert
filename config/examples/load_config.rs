use config::Config;

fn main() {
	dbg!(serde_json::from_str::<Config>(include_str!("config.example.json")).unwrap());
}