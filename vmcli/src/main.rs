use crate::{
    command::{address::Address, export::Export, import::Import, status::Status},
    error::VmCliError,
    executor::Runnable,
    proj_config::Config,
};
use clap::Parser;
use std::path::PathBuf;

pub mod command;
pub mod error;
pub mod executor;
pub mod logger;
pub mod proj_config;
pub mod utils;

pub static UNKONW: &str = "UNKONW";

#[derive(Parser, Debug)]
#[command(name = "vmcli", about, version)]
struct VmCli {
    #[command(subcommand)]
    command: Option<SubCommand>,

    #[arg(short, long, help = "Set VictoriaMetrics service address")]
    path: Option<String>,
}

#[derive(Parser, Debug)]
enum SubCommand {
    EXPORT(Export),
    IMPORT(Import),

    #[command(about = "Show Export/Import history info")]
    #[clap(alias = "i")]
    Info,
}

#[tokio::main]
async fn main() -> Result<(), VmCliError> {
    let confg_path = PathBuf::from("config");
    if !confg_path.exists() {
        vm_init().await?;
    }
    let config = Config::init_config().await?;
    logger::init(config.log)?;
    // 初始化状态配置
    init_status_config().await?;
    let cli = VmCli::parse();

    if let Some(path) = cli.path {
        Address::new(&path).run(&config.reqwest).await?;
    } else if let Some(command) = cli.command {
        match command {
            SubCommand::EXPORT(export) => export.run(&config.reqwest).await?,
            SubCommand::IMPORT(import) => import.run(&config.reqwest).await?,
            SubCommand::Info => {
                let status = Status::get_status().await?;
                status.pretty_print()?;
            }
        }
    }
    Ok(())
}

async fn init_status_config() -> Result<(), VmCliError> {
    let config_path = PathBuf::from("config/status_config.toml");

    // 如果配置文件不存在则执行初始化操作
    if !config_path.exists() {
        let status = Status::default();
        tokio::fs::write(config_path, toml::to_string(&status)?).await?;
    }
    Ok(())
}


async fn vm_init() -> Result<(), VmCliError> {
    log::info!("初始化配置文件");
    let base_config_path = PathBuf::from("config");
    if base_config_path.exists() {
        tokio::fs::remove_dir_all(&base_config_path).await?;
    }else{
        tokio::fs::create_dir(&base_config_path).await?;
    }
    // 初始化项目配置文件
    let config = Config::default();
    let config_path = base_config_path.join("proj_config.toml");
    tokio::fs::File::create(&config_path).await?;
    tokio::fs::write(&config_path, toml::to_string(&config)?).await?;

    // 初始化状态文件
    let status = Status::default();
    let status_path = base_config_path.join("status_config.toml");
    tokio::fs::File::create(&status_path).await?;
    tokio::fs::write(&status_path, toml::to_string(&status)?).await?;

    // 初始化导入记录文件
    tokio::fs::File::create(&base_config_path.join(".import_record.toml")).await?;
    Ok(())
}
