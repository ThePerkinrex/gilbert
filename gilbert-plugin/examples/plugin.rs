use gilbert_plugin::{init_plugin_fn, PluginBuilder, plugin::Plugin, sender::Sender};
use semver::Version;

struct P;

impl PluginBuilder for P {
    type Built<S: Sender<gilbert_plugin_api::GeneralPluginResponse<gilbert_plugin_api::plugin_proto::PluginResponse>>> = Self;

    fn build<S: Sender<gilbert_plugin_api::GeneralPluginResponse<gilbert_plugin_api::plugin_proto::PluginResponse>>>(self, _: S) -> Self::Built<S> {
        self
    }
}
impl Plugin for P {}

#[tokio::main]
async fn main() {
    init_plugin_fn(Version::new(0, 1, 0), |_: ()| P).await
}
