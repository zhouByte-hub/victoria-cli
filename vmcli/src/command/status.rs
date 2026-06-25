use crate::VmCliError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct Status {
    pub victoria_metrics_path: String,
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
            victoria_metrics_path: String::from("http://127.0.0.1:9428"),
            update_time: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            export: ExportStatus::default(),
            import: ImportStatus::default(),
        }
    }
}

impl Status {
    pub async fn get_status() -> Result<Status, VmCliError> {
        let content = tokio::fs::read_to_string(PathBuf::from("config/status_config.toml")).await?;
        let config: Status = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn pretty_print(&self) -> Result<(), VmCliError> {
        let status_json = json!({
            "victoria_metrics_path": self.victoria_metrics_path,
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
