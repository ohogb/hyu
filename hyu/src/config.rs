use crate::Result;
use color_eyre::eyre::ContextCompat;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(default)]
pub struct Config {
	pub keymap: String,
	pub card: std::path::PathBuf,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			keymap: String::from("us"),
			card: std::path::PathBuf::from("/dev/dri/card0"),
		}
	}
}

impl Config {
	pub fn read_from_config_file() -> Result<&'static Self> {
		let home_dir = std::env::home_dir().context("failed to get home dir")?;
		let home_dir = home_dir.to_str().context("home dir not utf-8")?;

		let config_path =
			std::path::PathBuf::from_iter([home_dir, ".config", "hyu", "config.json"]);
		let config_file =
			std::fs::read_to_string(config_path).unwrap_or_else(|_| String::from("{}"));

		let config = serde_json::from_str(&config_file)?;
		Ok(Box::leak(Box::new(config)))
	}
}
