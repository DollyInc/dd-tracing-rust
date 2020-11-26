use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Dd {
  pub service: String,
  pub env: String,
  pub version: String,
  #[serde(default = "default_host")]
  pub host: String,
  #[serde(default = "default_port")]
  pub port: String
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Metadata {
  pub env: String,
  pub image: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
  #[serde(default = "default_level")]
  pub level: String,
  pub prefix: String,
  pub metadata: Metadata,
  pub dd: Dd
}

fn default_host() -> String {
  "localhost".to_string()
}

fn default_port() -> String {
  "8126".to_string()
}

fn default_level() -> String {
  "info".to_string()
}