use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::UNKONW;
use crate::{
    command::status::Status, error::VmCliError, executor::Runnable, proj_config::ReqwestConfig,
    utils::tar_utils::TarUtils,
};
use clap::Parser;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::Client;
use std::env::home_dir;
use std::ffi::OsStr;
use tokio::time::Duration;

#[derive(Debug, Parser)]
#[command(
    name = "import",
    about = "Import data from the folder into VictoriaLogs"
)]
pub struct Import {
    #[arg(short, long, help = "Import data from the file into VictoriaLogs")]
    file: PathBuf,
}

impl Runnable for Import {
    fn run(&self, config: &ReqwestConfig) -> impl Future<Output = Result<(), VmCliError>> + Send {
        log::info!("开始执行导入任务...");
        self.import_run(config)
    }
}

impl Import {
    async fn recursive_decompress(
        file: &PathBuf,
        config: &ReqwestConfig,
        status: &Status,
        dest: &PathBuf,
        client: &Client,
        url: &str,
    ) -> Result<(), VmCliError> {
        let file_name = file
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or("Cannot get file name")?;

        if !file_name.ends_with(".tar.gz") {
            if !file_name.ends_with(".json") {
                let message =
                    VmCliError::VlogVmCliError("当前导入的内容不是一个合法的导入文件".into());
                return Err(message);
            }
            Import::import_file(file, client, url).await?;
        } else {
            let tar_path = TarUtils::decompress_tar(file, dest).await?;

            let mut read_dir = tokio::fs::read_dir(&tar_path).await?;
            while let Some(entry) = read_dir.next_entry().await? {
                let path = entry.path();
                // 使用Box::pin包装递归调用以避免无限大小的future
                Box::pin(Import::recursive_decompress(
                    &path, config, status, dest, client, url,
                ))
                .await?;
            }
            tokio::fs::remove_dir_all(&tar_path).await?;
        }
        Ok(())
    }

    async fn import_file(file: &PathBuf, client: &Client, url: &str) -> Result<(), VmCliError> {
        let file_name = file
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(UNKONW)
            .to_owned();
        // 根据文件名称判断文件是否已经上传过
        let file_hash = blake3::hash(file_name.as_bytes()).to_string();
        if Import::is_duplicate_upload(&file_hash).await? {
            return Ok(());
        };
        let content = tokio::fs::read_to_string(&file).await?;
        let response = client
            .post(url)
            .header("Content-Type", "application/stream+json")
            .body(content)
            .send()
            .await?;
        if response.status() != 200 {
            log::error!("{}", response.text().await?);
        } else {
            Import::update_import_record(&file_hash, &file_name).await?;
        }
        Ok(())
    }

    async fn import_run(&self, config: &ReqwestConfig) -> Result<(), VmCliError> {
        let status = Status::get_status().await?;

        let client = Arc::new(
            reqwest::Client::builder()
                .timeout(Duration::from_secs(config.global_timeout))
                .connect_timeout(Duration::from_secs(config.connect_timeout))
                .build()?,
        );

        let url = Arc::new(format!(
            "{}{}",
            status.victoria_metrics_path, config.victoria_metrics_import_path
        ));

        let temp_dir = Arc::new(
            home_dir()
                .ok_or("Failed to read the home directory")?
                .join(&config.download_path)
                .join(".temp"),
        );

        let first = TarUtils::decompress_tar(&self.file, &temp_dir).await?;
        let mut entries = tokio::fs::read_dir(&first).await?; // 当前目录
        // 得到子压缩包
        let mut files = Vec::new();
        while let Some(entry) = entries.next_entry().await? {
            let entry = entry;
            if entry.file_type().await?.is_file() {
                files.push(entry.path());
            }
        }
        let thread_count = std::thread::available_parallelism()?.get();
        let chunk_size = (files.len() as f64 / thread_count as f64).ceil() as usize;
        // 初始化进度条
        let multi_progress = MultiProgress::new();
        let progress_style = ProgressStyle::default_bar()
            .template("{msg} [{bar:40.cyan/blue}] {pos}/{len} ({eta})")?
            .progress_chars("#>-");

        let mut json_set = tokio::task::JoinSet::new();
        for chunk in files.chunks(chunk_size) {
            let chunk_vec = chunk.to_vec();
            // 独享配置
            let client_copy = Arc::clone(&client);
            let url_copy = Arc::clone(&url);
            let pb = multi_progress.add(ProgressBar::new(chunk_vec.len() as u64));
            let status_copy = status.clone();
            let config_copy = config.clone();
            let temp_dir_copy = Arc::clone(&temp_dir);
            pb.set_style(progress_style.clone());

            json_set.spawn(async move {
                for file_path in chunk_vec {
                    let filename = file_path
                        .file_name()
                        .ok_or("filename Error")?
                        .to_os_string();
                    pb.set_message(format!("Import Task [{:?}]", filename));
                    Import::recursive_decompress(
                        &file_path,
                        &config_copy,
                        &status_copy,
                        &temp_dir_copy,
                        &client_copy,
                        &url_copy,
                    )
                    .await?;
                    pb.inc(1);
                }
                Ok::<(), VmCliError>(())
            });
        }
        while let Some(res) = json_set.join_next().await {
            if let Err(e) = res {
                log::error!("导入出现错误：{}", e);
                std::process::exit(1);
            }
        }
        tokio::fs::remove_dir_all(&first).await?;
        log::info!("VictoriaLogs data import successfully!");
        Ok(())
    }

    async fn is_duplicate_upload(file_hash: &str) -> Result<bool, VmCliError> {
        let file_content = tokio::fs::read_to_string("config/.import_record.toml").await?;
        let records: HashMap<String, String> = toml::from_str(&file_content)?;
        Ok(records.contains_key(file_hash))
    }

    async fn update_import_record(file_hash: &str, file_name: &str) -> Result<(), VmCliError> {
        let file_path = "config/.import_record.toml";
        let file_content = tokio::fs::read_to_string(file_path).await?;

        let mut records: HashMap<String, String> = toml::from_str(&file_content)?;
        records.insert(file_hash.to_string(), file_name.to_string());

        tokio::fs::write(file_path, toml::to_string(&records)?.as_bytes()).await?;
        Ok(())
    }
}

#[cfg(test)]
mod import_test {
    use std::path::PathBuf;

    use crate::proj_config::ReqwestConfig;
    use crate::{command::import::Import, error::VmCliError};
    use tokio::time::Duration;

    #[tokio::test]
    async fn read_test() -> Result<(), VmCliError> {
        let import = Import {
            file: PathBuf::from("/Users/zhoujianing/vlogcli/a/2025-08-18T16:00:01Z.tar.gz"),
        };

        let value = ReqwestConfig {
            victoria_metrics_export_path: String::from("/select/logsql/query"),
            download_path: String::from("vlogcli"),
            victoria_metrics_import_path: String::from("/insert/jsonline"),
            global_timeout: 10,
            connect_timeout: 10,
            response_timeout: 10,
            idle_timeout: 10,
            max_idle: 10,
        };

        import.import_run(&value).await?;
        Ok(())
    }

    /**
        curl -X POST -H 'Content-Type: application/stream+json' --data-binary $'{"project_name":"otlp_service_A","serverity_text":"INFO","event_name":null,"_time":"2025-08-14T15:32:10Z","_msg":"Creating log with ID: 051b7e63-42bf-4268-a2af-598639a3199a","trace_id":"c38aad4833385782ca261b6048f77d1f","span_id":"6272fe7d9b2d8d39","diff":1,"time":1753854643032}
        {"project_name":"otlp_service_A","serverity_text":"INFO","event_name":null,"_time":"2025-08-14T15:32:10Z","_msg":"Creating log with ID: 051b7e63-42bf-4268-a2af-598639a3199a","trace_id":"c38aad4833385782ca261b6048f77d1f","span_id":"6272fe7d9b2d8d39","diff":1,"time":1753854643032}' 'http://43.139.97.119:9428/insert/jsonline'
    */
    #[tokio::test]
    async fn test() -> Result<(), VmCliError> {
        let client = reqwest::Client::new();
        let body = r#"{"_time":"2025-08-14T13:25:40.68555Z","_stream_id":"0000000000000000e934a84adb05276890d7f7bfcadabe92","_stream":"{}","_msg":"Error in event loop: I/O: Connection refused (os error 61)","serverity_text":"ERROR","service_name":"remote-worker","log_id":"log1755005380191333379wji","resource":"[{\"key\":\"service.name\",\"value\":\"remote-worker\"},{\"key\":\"telemetry.sdk.version\",\"value\":\"0.29.0\"},{\"key\":\"resourceType\",\"value\":\"logs\"},{\"key\":\"telemetry.sdk.name\",\"value\":\"opentelemetry\"},{\"key\":\"customerSystem\",\"value\":false},{\"key\":\"telemetry.sdk.language\",\"value\":\"rust\"}]"}"#;

        let response = client
            .post("http://43.139.97.119:9428/insert/jsonline")
            .header("Content-Type", "application/stream+json")
            .body(body)
            .timeout(Duration::from_secs(10))
            .send()
            .await?;

        println!("Status: {}", response.status());
        let text = response.text().await?;
        println!("Response: {}", text);

        Ok(())
    }
}
