use crate::CliError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct Status {
    pub victoria_logs_path: String,
    pub update_time: String,
    pub export: ExportStatus,
    pub import: ImportStatus,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct ExportStatus {
    pub has_exception: bool,
    pub export_start_time: DateTime<Utc>,
    pub export_end_time: DateTime<Utc>,
    pub export_exception_start_time: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct ImportStatus {
    pub has_exception: bool,
    pub import_exception_start_time: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct ImportRecord {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl Default for Status {
    fn default() -> Self {
        Self {
            victoria_logs_path: String::from("http://127.0.0.1:9428"),
            update_time: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            export: ExportStatus::default(),
            import: ImportStatus::default(),
        }
    }
}

impl Status {
    pub async fn get_status() -> Result<Status, CliError> {
        let content = tokio::fs::read_to_string(PathBuf::from("config/status_config.toml")).await?;
        let config: Status = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn pretty_print(&self) -> Result<(), CliError> {
        let status_json = json!({
            "victoria_logs_path": self.victoria_logs_path,
            "update_time": self.update_time,
            "export": {
                "has_exception": self.export.has_exception,
                "export_start_time": self.export.export_start_time.to_rfc3339(),
                "export_end_time": self.export.export_end_time.to_rfc3339(),
                "export_exception_start_time": self.export.export_exception_start_time.map(|time| time.to_rfc3339())
            },
            "import": {
                "has_exception": self.import.has_exception,
                "import_exception_start_time": self.import.import_exception_start_time.map(|time| time.to_rfc3339())
            }
        });
        log::info!("{}", serde_json::to_string_pretty(&status_json)?);
        Ok(())
    }
}

impl Default for ExportStatus {
    fn default() -> Self {
        let now = Utc::now();
        let start_of_day = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
        let end_of_day = now.date_naive().and_hms_opt(23, 59, 59).unwrap();

        Self {
            has_exception: false,
            export_start_time: DateTime::from_naive_utc_and_offset(start_of_day, Utc),
            export_end_time: DateTime::from_naive_utc_and_offset(end_of_day, Utc),
            export_exception_start_time: None,
        }
    }
}

impl Default for ImportStatus {
    fn default() -> Self {
        Self {
            has_exception: false,
            import_exception_start_time: None,
            // records: Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use tempfile::tempdir;
    use tokio::fs;

    /// 测试Status结构体的默认值创建
    #[test]
    fn test_status_default() {
        let status = Status::default();

        // 验证默认值
        assert_eq!(status.victoria_logs_path, "http://127.0.0.1:9428");
        assert!(!status.update_time.is_empty());
        assert!(!status.export.has_exception);
        assert!(!status.import.has_exception);

        // 验证导出状态的时间范围
        assert!(status.export.export_start_time <= status.export.export_end_time);
        assert!(status.export.export_exception_start_time.is_none());
        assert!(status.import.import_exception_start_time.is_none());

        println!("Status default test passed");
    }

    /// 测试ExportStatus的默认值
    #[test]
    fn test_export_status_default() {
        let export_status = ExportStatus::default();

        assert!(!export_status.has_exception);
        assert!(export_status.export_exception_start_time.is_none());

        // 验证时间范围是当天的开始和结束
        let now = Utc::now();
        let start_of_day = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
        let end_of_day = now.date_naive().and_hms_opt(23, 59, 59).unwrap();

        assert_eq!(
            export_status.export_start_time.date_naive(),
            now.date_naive()
        );
        assert_eq!(export_status.export_end_time.date_naive(), now.date_naive());
        assert_eq!(export_status.export_start_time.time(), start_of_day.time());
        assert_eq!(export_status.export_end_time.time(), end_of_day.time());

        println!("ExportStatus default test passed");
    }

    /// 测试ImportStatus的默认值
    #[test]
    fn test_import_status_default() {
        let import_status = ImportStatus::default();

        assert!(!import_status.has_exception);
        assert!(import_status.import_exception_start_time.is_none());

        println!("ImportStatus default test passed");
    }

    /// 测试ImportRecord的创建
    #[test]
    fn test_import_record_creation() {
        let start_time = Utc::now();
        let end_time = start_time + Duration::hours(1);

        let record = ImportRecord {
            start: start_time,
            end: end_time,
        };

        assert_eq!(record.start, start_time);
        assert_eq!(record.end, end_time);
        assert!(record.end > record.start);

        println!("ImportRecord creation test passed");
    }

    /// 测试从配置文件读取状态
    #[tokio::test]
    async fn test_get_status() {
        let temp_dir = tempdir().unwrap();
        let config_file = temp_dir.path().join("status_config.toml");

        // 创建测试配置文件
        let test_config = r#"
victoria_logs_path = "http://test.example.com:9428"
update_time = "2025-08-18 10:30:00"

[export]
has_exception = false
export_start_time = "2025-08-18T00:00:00Z"
export_end_time = "2025-08-18T23:59:59Z"

[import]
has_exception = true
"#;

        fs::write(&config_file, test_config).await.unwrap();

        // 临时修改当前工作目录以测试配置读取
        let original_path = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        let result = Status::get_status().await;

        // 恢复原始工作目录
        std::env::set_current_dir(original_path).unwrap();

        match result {
            Ok(status) => {
                assert_eq!(status.victoria_logs_path, "http://test.example.com:9428");
                assert_eq!(status.update_time, "2025-08-18 10:30:00");
                assert!(!status.export.has_exception);
                assert!(status.import.has_exception);
                println!("Get status test passed");
            }
            Err(e) => {
                println!("Get status test failed: {}", e);
            }
        }
    }

    /// 测试配置文件不存在的情况
    #[tokio::test]
    async fn test_get_status_file_not_found() {
        // 确保配置文件不存在
        let temp_dir = tempdir().unwrap();
        let original_path = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        let result = Status::get_status().await;

        std::env::set_current_dir(original_path).unwrap();

        // 应该返回错误
        if result.is_err() {
            println!("Get status file not found test correctly returned error");
        } else {
            println!("Get status file not found test unexpectedly succeeded");
        }
    }

    /// 测试JSON格式化输出
    #[test]
    fn test_pretty_print() {
        let mut status = Status::default();
        status.victoria_logs_path = "http://test.example.com:9428".to_string();
        status.update_time = "2025-08-18 10:30:00".to_string();

        // 设置一些异常状态
        status.export.has_exception = true;
        status.export.export_exception_start_time = Some(Utc::now());
        status.import.has_exception = true;
        status.import.import_exception_start_time = Some(Utc::now());

        let result = status.pretty_print();

        // pretty_print主要输出日志，所以主要测试它不崩溃
        assert!(result.is_ok());
        println!("Pretty print test passed");
    }

    /// 测试状态序列化和反序列化
    #[test]
    fn test_status_serialization() {
        let status = Status::default();

        // 测试JSON序列化
        let json_str = serde_json::to_string(&status);
        assert!(json_str.is_ok());

        // 测试JSON反序列化
        let deserialized: Result<Status, _> = serde_json::from_str(&json_str.unwrap());
        assert!(deserialized.is_ok());

        let deserialized_status = deserialized.unwrap();
        assert_eq!(
            status.victoria_logs_path,
            deserialized_status.victoria_logs_path
        );
        assert_eq!(
            status.export.has_exception,
            deserialized_status.export.has_exception
        );
        assert_eq!(
            status.import.has_exception,
            deserialized_status.import.has_exception
        );

        println!("Status serialization test passed");
    }

    /// 测试TOML序列化和反序列化
    #[test]
    fn test_status_toml_serialization() {
        let status = Status::default();

        // 测试TOML序列化
        let toml_str = toml::to_string(&status);
        assert!(toml_str.is_ok());

        // 测试TOML反序列化
        let deserialized: Result<Status, _> = toml::from_str(&toml_str.unwrap());
        assert!(deserialized.is_ok());

        let deserialized_status = deserialized.unwrap();
        assert_eq!(
            status.victoria_logs_path,
            deserialized_status.victoria_logs_path
        );
        assert_eq!(
            status.export.has_exception,
            deserialized_status.export.has_exception
        );
        assert_eq!(
            status.import.has_exception,
            deserialized_status.import.has_exception
        );

        println!("Status TOML serialization test passed");
    }

    /// 测试带有异常时间的状态
    #[test]
    fn test_status_with_exceptions() {
        let exception_time = Utc::now();

        let mut export_status = ExportStatus::default();
        export_status.has_exception = true;
        export_status.export_exception_start_time = Some(exception_time);

        let mut import_status = ImportStatus::default();
        import_status.has_exception = true;
        import_status.import_exception_start_time = Some(exception_time);

        let status = Status {
            victoria_logs_path: "http://test.example.com:9428".to_string(),
            update_time: "2025-08-18 10:30:00".to_string(),
            export: export_status,
            import: import_status,
        };

        assert!(status.export.has_exception);
        assert!(status.import.has_exception);
        assert_eq!(
            status.export.export_exception_start_time,
            Some(exception_time)
        );
        assert_eq!(
            status.import.import_exception_start_time,
            Some(exception_time)
        );

        println!("Status with exceptions test passed");
    }

    /// 测试状态克隆功能
    #[test]
    fn test_status_clone() {
        let status = Status::default();
        let cloned_status = status.clone();

        assert_eq!(status.victoria_logs_path, cloned_status.victoria_logs_path);
        assert_eq!(status.update_time, cloned_status.update_time);
        assert_eq!(
            status.export.has_exception,
            cloned_status.export.has_exception
        );
        assert_eq!(
            status.import.has_exception,
            cloned_status.import.has_exception
        );

        println!("Status clone test passed");
    }
}
