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
  #[serde(default = "default_sample_rate")]
  pub sample_rate: f32,
  pub metadata: Metadata,
  pub dd: Dd
}

impl Config {
  pub fn validate(&self) -> Result<(), &'static str> {
    if self.level.is_empty() {
      return Err("Invalid level.")
    }
    if self.sample_rate < 0f32 || self.sample_rate > 1f32 {
      return Err("Invalid sample rate.")
    }
    if self.dd.host.is_empty() {
      return Err("Invalid host.")
    }
    if self.dd.port.is_empty() {
      return Err("Invalid port.")
    }
    Ok(())
  }
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

fn default_sample_rate() -> f32 {
  1f32
}