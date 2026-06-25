use crate::utils::byte_utils::ByteUtils;
use crate::utils::tar_utils::TarUtils;
use crate::{
    Status, UNKONW, error::VmCliError, executor::Runnable, proj_config::ReqwestConfig,
    utils::time_utils::TimeUtils,
};
use chrono::{DateTime, Utc};
use clap::Parser;
use futures_util::StreamExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::Response;
use std::collections::HashMap;
use std::env::home_dir;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Parser)]
#[command(name = "export", about = "Export data from VictoriaMetrics")]
pub struct Export {
    #[arg(
        short,
        long,
        help = "Filter criteria for exporting data from VictoriaMetrics",
        default_value = "{__name__!=\"\"}"
    )]
    query: Option<String>,

    #[arg(short, long, help = "Scope - Start Time from VictoriaMetrics export")]
    start_time: Option<DateTime<Utc>>,

    #[arg(short, long, help = "Scope - End Time from VictoriaMetrics export")]
    end_time: Option<DateTime<Utc>>,

    #[arg(short = 't', long, default_value = "60", help = "Time step in seconds")]
    step: i64,

    #[arg(short = 'i', long, help = "Responsible organization ID for export")]
    id: String,

    #[arg(short = 'n', long, help = "Responsible organization name for export")]
    name: String,
}

impl Runnable for Export {
    fn run(&self, config: &ReqwestConfig) -> impl Future<Output = Result<(), VmCliError>> + Send {
        log::info!("开始执行导出任务...");
        self.export_run(config)
    }
}

impl Export {
    pub async fn export_run(&self, config: &ReqwestConfig) -> Result<(), VmCliError> {
        let mut status = Status::get_status().await?;
        let form_data = self.analysis_export_condition(&mut status).await?;

        let tar_path = self
            .download(
                &mut status.export.export_start_time,
                &mut status.export.export_end_time,
                &status.victoria_metrics_path,
                &form_data,
                config,
            )
            .await?;
        log::info!("导出的位置为：{:?}", tar_path);
        self.update_status_file(&mut status).await?;
        Ok(())
    }

    async fn analysis_export_condition(
        &self,
        status: &mut Status,
    ) -> Result<HashMap<&'static str, String>, VmCliError> {
        let mut condition = HashMap::new();
        let exporter = &mut status.export;

        if exporter.has_exception {
            // todo: 异常情况处理
        } else {
            let yesterday = Utc::now().date_naive() - chrono::Duration::days(1);
            let start_of_day = yesterday.and_hms_opt(0, 0, 0).unwrap();
            let end_of_day = yesterday.and_hms_opt(23, 59, 59).unwrap();

            if let Some(query) = self.query.as_ref() {
                condition.insert("match[]", query.clone());
            }
            if let Some(start_time) = self.start_time.as_ref() {
                exporter.export_start_time = *start_time;
                condition.insert("start", TimeUtils::format_time_3339_opts(&start_time));
            } else {
                let default_start_time: DateTime<Utc> =
                    DateTime::from_naive_utc_and_offset(start_of_day, Utc);
                exporter.export_start_time = default_start_time;
                condition.insert(
                    "start",
                    TimeUtils::format_time_3339_opts(&default_start_time),
                );
            }
            if let Some(end_time) = self.end_time.as_ref() {
                exporter.export_end_time = *end_time;
                condition.insert("end", TimeUtils::format_time_3339_opts(&end_time));
            } else {
                let default_end_time: DateTime<Utc> =
                    DateTime::from_naive_utc_and_offset(end_of_day, Utc);
                exporter.export_end_time = default_end_time;
                condition.insert("end", TimeUtils::format_time_3339_opts(&default_end_time));
            }
        }
        Ok(condition)
    }

    async fn update_status_file(&self, status: &mut Status) -> Result<(), VmCliError> {
        status.update_time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let content = toml::to_string(status)?;
        tokio::fs::write(PathBuf::from("config/status_config.toml"), content).await?;
        Ok(())
    }

    async fn download(
        &self,
        start_time: &DateTime<Utc>,
        end_time: &DateTime<Utc>,
        path: &str,
        data: &HashMap<&'static str, String>,
        config: &ReqwestConfig,
    ) -> Result<PathBuf, VmCliError> {
        let client = Arc::new(
            reqwest::Client::builder()
                .timeout(Duration::from_secs(config.global_timeout))
                .connect_timeout(Duration::from_secs(config.connect_timeout))
                .tcp_nodelay(true) // 禁用Nagle算法，减少延迟
                .pool_idle_timeout(Duration::from_secs(config.idle_timeout)) // 连接池空闲超时
                .pool_max_idle_per_host(config.max_idle) // 每个主机最大空闲连接数
                .build()?,
        );
        let url = Arc::new(format!("{}{}", path, config.victoria_metrics_export_path));
        let mut home = home_dir()
            .ok_or("Failed to read the home directory")?
            .join(&config.download_path);
        if !home.exists() {
            tokio::fs::create_dir_all(&home).await?;
        }
        let time_list = TimeUtils::split_time_range(start_time, end_time, self.step);
        let multi_progress = MultiProgress::new();
        let progress_style = ProgressStyle::default_bar()
            .template("{msg} [{bar:40.cyan/blue}] {pos}/{len} ({eta})")?
            .progress_chars("#>-");

        let thread_count = std::thread::available_parallelism()?.get() * 2;
        let chunk_size = (time_list.len() as f64 / thread_count as f64).ceil() as usize;
        let mut json_set = tokio::task::JoinSet::new();
        // 多线程开始
        let mut total_bytes = 0;
        for item in time_list.chunks(chunk_size) {
            let chunk_vec = item.to_vec();
            let mut data_copy = data.clone();
            let (s, e) = TimeUtils::get_time_bound(&item)?;
            data_copy.insert("start", TimeUtils::format_time_3339_opts(&s));
            data_copy.insert("end", TimeUtils::format_time_3339_opts(&e));

            let pb = multi_progress.add(ProgressBar::new(chunk_vec.len() as u64));
            pb.set_style(progress_style.clone());

            // 每个线程有一个副本独享数据
            let client_copy = Arc::clone(&client);
            let url_copy = Arc::clone(&url);
            let tar_path = home.join(format!("{}.tar.gz", TimeUtils::format_time_3339_opts(&s)));
            let config_copy = config.clone();
            json_set.spawn(async move {
                let mut list = Vec::new();
                let mut bytes = 0;
                for (s_time, e_time) in chunk_vec.iter() {
                    let temp_s_time_str = TimeUtils::format_time_3339_opts(s_time);
                    let temp_e_time_str = TimeUtils::format_time_3339_opts(e_time);

                    data_copy.insert("start", temp_s_time_str.clone());
                    data_copy.insert("end", temp_e_time_str.clone());

                    pb.set_message(format!(
                        "Export Task [{}-{}]",
                        &temp_s_time_str, &temp_e_time_str
                    ));
                    let response = client_copy
                        .post(&*url_copy)
                        .timeout(Duration::from_secs(config_copy.response_timeout))
                        .header("Accept", "application/json")
                        .header("Connection", "keep-alive")
                        .form(&data_copy)
                        .send()
                        .await?;
                    // 当前时间段没有数据
                    if response.content_length() == Some(0) {
                        pb.inc(1);
                        continue;
                    }
                    if response.status().is_success() {
                        let filename = data_copy
                            .get("start")
                            .cloned()
                            .unwrap_or(UNKONW.to_string());
                        let file_path = Export::save_data(
                            format!("{}.json", filename).as_str(),
                            &tar_path.parent().ok_or("Parent Error")?,
                            response,
                        )
                        .await?;
                        list.push(file_path.0);
                        bytes += file_path.1;
                        pb.inc(1);
                    } else {
                        let message = format!("导出请求错误： {}", response.text().await?);
                        log::error!("{}", message);
                        return Err(VmCliError::VlogVmCliError(message));
                    }
                }
                TarUtils::compress_file_to_tar(&list, &tar_path).await?;
                list.clear();
                Ok::<usize, VmCliError>(bytes)
            });
        }
        while let Some(result) = json_set.join_next().await {
            if let Ok(ok) = result {
                total_bytes += match ok {
                    Err(e) => {
                        log::error!("导出失败：{:?}", e);
                        tokio::fs::remove_dir_all(&home).await?;
                        std::process::exit(1);
                    }
                    Ok(count) => count,
                };
            } else {
                log::error!("导出失败");
                tokio::fs::remove_dir_all(&home).await?;
                std::process::exit(1);
            }
        }
        // 生成 mate 数据文件
        self.generator_meta_file(&home).await?;
        // 汇总压缩包
        log::info!("正在压缩数据，请稍后...");
        let mut read_dir = tokio::fs::read_dir(&home).await?;
        let mut list = Vec::new();
        while let Some(entry) = read_dir.next_entry().await? {
            let path = entry.path();
            // 忽略隐藏文件（以点开头的文件）
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.starts_with('.') {
                    continue;
                }
                list.push(path);
            }
        }
        home = home.join(format!("{}.tar.gz", Utc::now().date_naive()));
        TarUtils::compress_file_to_tar(&list, &home).await?;
        log::info!(
            "压缩包汇总完成，一共导出 {}MB 的指标数据",
            ByteUtils::bytes_to_mb(total_bytes)
        );
        Ok(home)
    }
}

impl Export {
    async fn save_data(
        filename: &str,
        download_path: &Path,
        response: Response,
    ) -> Result<(PathBuf, usize), VmCliError> {
        let mut temp_path = download_path.join(".temp");
        if !temp_path.exists() {
            tokio::fs::create_dir_all(&temp_path).await?;
        }
        temp_path = temp_path.join(filename);
        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&temp_path)
            .await?;
        let mut writer_buffer = tokio::io::BufWriter::new(&mut file);

        let mut stream = response.bytes_stream();
        let mut bytes = 0;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            bytes += chunk.len();
            writer_buffer.write_all(&chunk).await?;
        }
        writer_buffer.flush().await?;
        Ok((temp_path, bytes))
    }

    async fn generator_meta_file(&self, home: &PathBuf) -> Result<(), VmCliError> {
        let meta_path = home.join("meta.json");
        let meta = serde_json::json!({
            "org_id": self.id,
            "org_name": self.name,
            "export_time": Utc::now().to_rfc3339(),
        });
        tokio::fs::write(&meta_path, serde_json::to_string(&meta)?).await?;
        Ok(())
    }
}

#[cfg(test)]
mod test {

    use crate::{command::export::Export, error::VmCliError, proj_config::ReqwestConfig};
    use chrono::{DateTime, Utc};
    #[tokio::test]
    async fn download_test() -> Result<(), VmCliError> {
        let value = ReqwestConfig {
            victoria_metrics_export_path: String::from("/select/logsql/query"),
            download_path: String::from("vmcli"),
            victoria_metrics_import_path: String::from("/insert/jsonline"),
            global_timeout: 10,
            connect_timeout: 10,
            response_timeout: 10,
            idle_timeout: 10,
            max_idle: 10,
        };

        let export = Export {
            query: Some(String::from("*")),
            start_time: None,
            end_time: None,
            step: 60,
            id: String::from("1"),
            name: String::from("test"),
        };
        export.export_run(&value).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test() -> Result<(), VmCliError> {
        // 解析时间字符串
        let start = DateTime::parse_from_rfc3339("2025-08-18T16:01:02Z")
            .expect("Invalid start time format")
            .with_timezone(&Utc);
        let end = DateTime::parse_from_rfc3339("2025-08-18T16:01:04Z")
            .expect("Invalid end time format")
            .with_timezone(&Utc);

        let value = ReqwestConfig {
            victoria_metrics_export_path: String::from("/select/logsql/query"),
            download_path: String::from("vmcli"),
            victoria_metrics_import_path: String::from("/insert/jsonline"),
            global_timeout: 600,
            connect_timeout: 300,
            response_timeout: 600,
            idle_timeout: 600,
            max_idle: 10,
        };

        let export = Export {
            query: Some(String::from("*")),
            start_time: Some(start),
            end_time: Some(end),
            step: 1,
            id: String::from("1"),
            name: String::from("test"),
        };
        export.export_run(&value).await?;
        Ok(())
    }
}
