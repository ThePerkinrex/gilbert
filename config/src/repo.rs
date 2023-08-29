use std::{path::{PathBuf, Component}, collections::HashMap};

use serde::{Deserialize, de::Expected};
use target_lexicon::Triple;

#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct Repository {
	pub plugins: HashMap<String, Plugin>
}

#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct Plugin {
	args: Vec<String>,
	binaries: HashMap<Triple, String>
}

// #[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq, Clone)]
// #[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
// #[serde(tag = "source")]
// #[serde(rename_all = "snake_case")]
// pub enum PluginSource {
// 	Fs {
// 		/// Relative path withou ..
// 		#[serde(deserialize_with = "relative_path_de")]
// 		path: PathBuf
// 	},
// 	Web {

// 	}
// }

// fn relative_path_de<'de, D>(de: D) -> Result<PathBuf, D::Error> where D: serde::Deserializer<'de> {
// 	let path = PathBuf::deserialize(de)?;
// 	if path.is_absolute() {
// 		Err(<D::Error as serde::de::Error>::invalid_type(serde::de::Unexpected::Other("absolute path"), &"relative path without .."))
// 	}else if path.components().any(|x| x == Component::ParentDir){
// 		Err(<D::Error as serde::de::Error>::invalid_value(serde::de::Unexpected::Other("Path with .."), &"relative path without .."))
// 	}else{
// 		Ok(path)
// 	}
// }