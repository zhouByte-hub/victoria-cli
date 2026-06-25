use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::VmCliError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub log: CliLogger,
    pub reqwest: ReqwestConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CliLogger {
    pub path: String,
    pub level: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReqwestConfig {
    pub victoria_metrics_export_path: String,
    pub download_path: String,
    pub victoria_metrics_import_path: String,
    pub global_timeout: u64,
    pub connect_timeout: u64,
    pub response_timeout: u64,
    pub idle_timeout: u64,
    pub max_idle: usize,
}

impl Config {
    pub async fn init_config() -> Result<Self, VmCliError> {
        let content = tokio::fs::read_to_string(PathBuf::from("config/proj_config.toml")).await?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            log: CliLogger::default(),
            reqwest: ReqwestConfig::default(),
        }
    }
}

impl Default for CliLogger {
    fn default() -> Self {
        Self {
            path: String::from("./logs"),
            level: String::from("info"),
        }
    }
}

impl Default for ReqwestConfig {
    fn default() -> Self {
        Self {
            victoria_metrics_export_path: String::from("/api/v1/export"),
            download_path: String::from("vmcli"),
            victoria_metrics_import_path: String::from("/api/v1/import"),
            global_timeout: 600,
            connect_timeout: 300,
            response_timeout: 120,
            idle_timeout: 30,
            max_idle: 0,
        }
    }
}

#[cfg(test)]
mod config_test {

    use crate::{error::VmCliError, proj_config::Config};
    use std::path::PathBuf;

    #[tokio::test]
    async fn test() -> Result<(), VmCliError> {
        let content = tokio::fs::read_to_string(PathBuf::from("config/proj_config.toml")).await?;
        let config: Config = toml::from_str(&content)?;
        println!("{:?}", config);
        Ok(())
    }
}
