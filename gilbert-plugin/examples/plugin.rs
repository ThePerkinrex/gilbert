use gilbert_plugin::{init_plugin_fn, Plugin};
use semver::Version;

#[derive(Debug, serde::Deserialize)]
struct Config {}

struct P;

impl Plugin for P {}

#[tokio::main]
async fn main() {
	init_plugin_fn(Version::new(0, 1, 0), |_: Config| P).await
}