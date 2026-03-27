use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
  pub printer: PrinterConfig,
  pub server: Option<ServerConfig>,
}

#[derive(Debug, Deserialize)]
pub struct PrinterConfig {
  pub ip: String,
  pub serial: String,
  pub access_code: String,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
  #[serde(default = "default_host")]
  pub host: String,
  #[serde(default = "default_port")]
  pub port: u16,
}

fn default_host() -> String {
  "0.0.0.0".into()
}
fn default_port() -> u16 {
  3000
}

impl Config {
  pub fn load() -> Result<Self> {
    let config_path =
      std::env::var("CONFIG_PATH").unwrap_or_else(|_| "/app/config/config.toml".into());

    let text = std::fs::read_to_string(&config_path)
      .with_context(|| format!("failed to read config file: {config_path}"))?;

    let mut config: Config = toml::from_str(&text).context("failed to parse config.toml")?;

    // Environment variable overrides — individual fields can be overridden
    // without touching the file. Useful for secrets in container deployments.
    if let Ok(v) = std::env::var("PRINTER_IP") {
      config.printer.ip = v;
    }
    if let Ok(v) = std::env::var("PRINTER_SERIAL") {
      config.printer.serial = v;
    }
    if let Ok(v) = std::env::var("PRINTER_ACCESS_CODE") {
      config.printer.access_code = v;
    }
    if let Ok(v) = std::env::var("SERVER_HOST") {
      if let Some(ref mut server_config) = config.server {
        server_config.host = v;
      } else {
        config.server = Some(ServerConfig {
          host: v,
          port: default_port(),
        });
      }
    }
    if let Ok(v) = std::env::var("SERVER_PORT") {
      let port = v.parse().context("SERVER_PORT must be a valid integer")?;
      if let Some(ref mut server_config) = config.server {
        server_config.port = port;
      } else {
        config.server = Some(ServerConfig {
          host: default_host(),
          port,
        });
      }
    }

    Ok(config)
  }

  pub fn server_addr(&self) -> String {
    let default_server = ServerConfig {
      host: default_host(),
      port: default_port(),
    };
    let server = self.server.as_ref().unwrap_or(&default_server);
    format!("{}:{}", server.host, server.port)
  }
}
