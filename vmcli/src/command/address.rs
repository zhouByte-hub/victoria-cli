use crate::proj_config::ReqwestConfig;
use crate::{command::status::Status, error::VmCliError, executor::Runnable};
use std::path::PathBuf;
use tokio::net::TcpStream;
use tokio::time::{Duration, timeout};
use url::Url;

pub struct Address {
    path: String,
}

impl Runnable for Address {
    fn run(
        &self,
        _config: &ReqwestConfig,
    ) -> impl Future<Output = Result<(), crate::error::VmCliError>> + Send {
        self.address_run()
    }
}

impl Address {
    pub fn new(path: &str) -> Self {
        Address {
            path: path.to_string(),
        }
    }
}

impl Address {
    pub async fn address_run(&self) -> Result<(), VmCliError> {
        log::info!("Update VictoriaMetrics server address");
        if Url::parse(&self.path).is_err() {
            return Err(VmCliError::InvalidUrl(self.path.clone()));
        }
        if !self.check_tcp_reachable().await? {
            return Err(VmCliError::TcpNotReachable(self.path.clone()));
        }
        let mut content =
            tokio::fs::read_to_string(PathBuf::from("config/status_config.toml")).await?;
        let mut config: Status = toml::from_str(&content)?;

        config.victoria_metrics_path = self.path.clone();
        config.update_time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        content = toml::to_string(&config)?;
        tokio::fs::write(PathBuf::from("config/status_config.toml"), content).await?;
        log::info!("VictoriaMetrics server address updated successfully");
        Ok(())
    }

    async fn check_tcp_reachable(&self) -> Result<bool, VmCliError> {
        let url = Url::parse(&self.path)?;
        let host = url.host_str().unwrap_or("127.0.0.1");
        let port = url.port().unwrap_or(9428);

        let path = format!("{}:{}", host, port);
        let check_result = timeout(Duration::from_secs(5), TcpStream::connect(path))
            .await
            .is_ok();
        Ok(check_result)
    }
}
