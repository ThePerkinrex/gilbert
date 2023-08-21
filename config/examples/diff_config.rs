use config::Config;
use diff::Diff;

fn main() {
	let config1 = serde_json::from_str::<Config>(include_str!("config.example.json")).unwrap();
	let config2 = serde_json::from_str::<Config>(include_str!("config2.example.json")).unwrap();
	let diff = config1.diff(&config2);
	let diff_json = serde_json::to_string_pretty(&diff).unwrap();
	println!("{diff_json}")
}