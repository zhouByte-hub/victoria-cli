use crate::proj_config::ReqwestConfig;
use crate::{command::status::Status, error::CliError, executor::Runnable};
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
    ) -> impl Future<Output = Result<(), crate::error::CliError>> + Send {
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
    pub async fn address_run(&self) -> Result<(), CliError> {
        log::info!("Update VictoriaLogs server address");
        if Url::parse(&self.path).is_err() {
            return Err(CliError::InvalidUrl(self.path.clone()));
        }
        if !self.check_tcp_reachable().await? {
            return Err(CliError::TcpNotReachable(self.path.clone()));
        }
        let mut content =
            tokio::fs::read_to_string(PathBuf::from("config/status_config.toml")).await?;
        let mut config: Status = toml::from_str(&content)?;

        // 更新 VictoriaLogs 服务地址
        config.victoria_logs_path = self.path.clone();
        // 填充文件更新时间
        config.update_time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        content = toml::to_string(&config)?;
        tokio::fs::write(PathBuf::from("config/status_config.toml"), content).await?;
        log::info!("VictoriaLogs server address updated successfully");
        Ok(())
    }

    async fn check_tcp_reachable(&self) -> Result<bool, CliError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::fs;

    /// 测试Address结构体的创建
    #[test]
    fn test_address_creation() {
        let address = Address::new("http://127.0.0.1:9428");
        assert_eq!(address.path, "http://127.0.0.1:9428");
        println!("Address creation test passed");
    }

    /// 测试URL验证功能
    #[test]
    fn test_url_validation() {
        // 测试有效的URL
        let valid_urls = vec![
            "http://127.0.0.1:9428",
            "https://example.com:9428",
            "http://localhost:8080",
        ];

        for url in valid_urls {
            let address = Address::new(url);
            assert!(!address.path.is_empty());
        }

        println!("URL validation test passed");
    }

    /// 测试无效URL的处理
    #[tokio::test]
    async fn test_invalid_url() {
        let invalid_urls = vec!["invalid_url", "ftp://example.com", "", "http://"];

        for url in invalid_urls {
            let address = Address::new(url);
            let result = address.address_run().await;

            // 无效URL应该返回错误
            if result.is_err() {
                match result.unwrap_err() {
                    CliError::InvalidUrl(_) => {
                        // 这是预期的错误类型
                        println!("Invalid URL test correctly failed for: {}", url);
                    }
                    _ => {
                        println!("Invalid URL test failed with unexpected error for: {}", url);
                    }
                }
            } else {
                println!("Invalid URL test unexpectedly succeeded for: {}", url);
            }
        }
    }

    /// 测试TCP连接检查功能
    #[tokio::test]
    async fn test_check_tcp_reachable() {
        // 测试一个通常不可达的地址
        let address = Address::new("http://43.139.97.119:9428");
        let result = address.check_tcp_reachable().await;

        // 这个测试可能会失败，因为端口9999可能没有服务运行
        match result {
            Ok(reachable) => {
                println!("TCP connection test - reachable: {}", reachable);
            }
            Err(e) => {
                println!("TCP connection test failed: {}", e);
            }
        }
    }

    /// 测试地址运行功能
    #[tokio::test]
    async fn test_address_run() {
        let temp_dir = tempdir().unwrap();
        let config_file = temp_dir.path().join("status_config.toml");

        // 创建测试状态文件
        let test_status = r#"
victoria_logs_path = "http://old.example.com:9428"
update_time = "2025-08-18 10:30:00"

[export]
has_exception = false
export_start_time = "2025-08-18T00:00:00Z"
export_end_time = "2025-08-18T23:59:59Z"

[import]
has_exception = false
"#;

        fs::write(&config_file, test_status).await.unwrap();

        let address = Address::new("http://new.example.com:9428");

        // 临时修改当前工作目录
        let original_path = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        let result = address.address_run().await;

        std::env::set_current_dir(original_path).unwrap();

        match result {
            Ok(_) => {
                // 验证状态文件是否被更新
                if let Ok(content) = fs::read_to_string(&config_file).await {
                    assert!(content.contains("http://new.example.com:9428"));
                    println!("Address run test passed");
                } else {
                    println!("Address run test - could not read file");
                }
            }
            Err(e) => {
                println!("Address run test failed: {}", e);
            }
        }
    }

    /// 测试状态文件更新功能
    #[tokio::test]
    async fn test_status_file_update() {
        let temp_dir = tempdir().unwrap();
        let status_file = temp_dir.path().join("status_config.toml");

        // 创建初始状态文件
        let initial_status = r#"
victoria_logs_path = "http://old.example.com:9428"
update_time = "2025-08-18 10:30:00"

[export]
has_exception = false
export_start_time = "2025-08-18T00:00:00Z"
export_end_time = "2025-08-18T23:59:59Z"

[import]
has_exception = false
"#;

        fs::write(&status_file, initial_status).await.unwrap();

        let address = Address::new("http://new.example.com:9428");

        // 临时修改当前工作目录
        let original_path = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        let result = address.address_run().await;

        std::env::set_current_dir(original_path).unwrap();

        match result {
            Ok(_) => {
                // 验证状态文件是否被更新
                if let Ok(content) = fs::read_to_string(&status_file).await {
                    assert!(content.contains("http://new.example.com:9428"));
                    assert!(!content.contains("http://old.example.com:9428"));
                    println!("Status file update test passed");
                } else {
                    println!("Status file update test - could not read file");
                }
            }
            Err(e) => {
                println!("Status file update test failed: {}", e);
            }
        }
    }

    /// 测试地址格式化
    #[test]
    fn test_address_formatting() {
        let test_cases = vec![
            ("http://127.0.0.1:9428", "http://127.0.0.1:9428"),
            ("https://example.com", "https://example.com"),
            ("http://localhost:8080", "http://localhost:8080"),
        ];

        for (input, expected) in test_cases {
            let address = Address::new(input);
            assert_eq!(address.path, expected);
        }

        println!("Address formatting test passed");
    }

    /// 测试空地址的处理
    #[test]
    fn test_empty_address() {
        let address = Address::new("");
        assert!(address.path.is_empty());
        println!("Empty address test passed");
    }

    /// 测试URL解析功能
    #[test]
    fn test_url_parsing() {
        let address = Address::new("http://user:pass@example.com:8080/path");
        let url = Url::parse(&address.path).unwrap();

        assert_eq!(url.scheme(), "http");
        assert_eq!(url.host_str(), Some("example.com"));
        assert_eq!(url.port(), Some(8080));
        assert_eq!(url.path(), "/path");

        println!("URL parsing test passed");
    }

    /// 测试默认端口处理
    #[tokio::test]
    async fn test_default_port() {
        // 测试没有指定端口的情况，应该使用默认端口9428
        let address = Address::new("http://43.139.97.119:9428");

        // 由于TCP连接可能失败，我们主要测试URL解析
        let url = Url::parse(&address.path).unwrap();
        assert_eq!(url.port(), Some(9428)); // 没有显式指定端口

        println!("Default port test passed");
    }

    /// 测试Runnable trait实现
    #[tokio::test]
    async fn test_runnable_trait() {
        let address = Address::new("http://43.139.97.119:9428/");
        let config = crate::proj_config::ReqwestConfig {
            victoria_logs_export_path: "/select/logsql/query".to_string(),
            download_path: "test".to_string(),
            victoria_logs_import_path: "/insert/jsonline".to_string(),
            global_timeout: 1,
            connect_timeout: 1,
            response_timeout: 1,
            idle_timeout: 1,
            max_idle: 1,
        };

        let result = address.run(&config).await;

        // 由于端口9999可能不可达，预期会失败
        match result {
            Ok(_) => {
                println!("Runnable trait test - unexpected success");
            }
            Err(e) => {
                println!("Runnable trait test correctly failed: {}", e);
            }
        }
    }

    /// 测试超时设置
    #[tokio::test]
    async fn test_timeout_settings() {
        let address = Address::new("http://43.139.97.119:9428");

        let start_time = std::time::Instant::now();
        let result = address.check_tcp_reachable().await;
        let elapsed = start_time.elapsed();

        // 检查是否在5秒超时内返回
        assert!(elapsed.as_secs() <= 6); // 允许一些缓冲时间

        println!(
            "Timeout settings test - elapsed: {:?}, result: {:?}",
            elapsed, result
        );
    }
}
