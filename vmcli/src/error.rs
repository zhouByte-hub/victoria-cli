use std::error::Error;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VmCliError {
    #[error("IO Exception: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Toml Deserialize Exception: {0}")]
    TomlDeserializeError(#[from] toml::de::Error),

    #[error("Toml Serialize Exception: {0}")]
    TomlSerializeError(#[from] toml::ser::Error),

    #[error("Url Parse Exception: {0}")]
    InvalidUrl(String),

    #[error("Tcp Not Reachable: {0}")]
    TcpNotReachable(String),

    #[error("Elapsed: {0}")]
    Elapsed(#[from] tokio::time::error::Elapsed),

    #[error("FlexiLoggerError: {0}")]
    FlexiLoggerError(#[from] flexi_logger::FlexiLoggerError),

    #[error("Url Parse Error: {0}")]
    ParseError(#[from] url::ParseError),

    #[error("Reqwest Error: {0}")]
    ReqwestError(reqwest::Error),

    #[error("VlogCli Error: {0}")]
    VlogVmCliError(String),

    #[error("SerdeJsonError: {0}")]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("StdError: {0}")]
    StdError(#[from] Box<dyn Error + Send + Sync + 'static>),

    #[error("JoinError: {0}")]
    JsonError(#[from] tokio::task::JoinError),

    #[error("TemplateError: {0}")]
    TemplateError(#[from] indicatif::style::TemplateError),

    #[error("TokioSendError: {0}")]
    TokioSendError(String),
}

impl From<&str> for VmCliError {
    fn from(err: &str) -> Self {
        VmCliError::VlogVmCliError(err.to_string())
    }
}

impl From<tokio::sync::mpsc::error::SendError<PathBuf>> for VmCliError {
    fn from(e: tokio::sync::mpsc::error::SendError<PathBuf>) -> Self {
        VmCliError::TokioSendError(format!("Could not send path: {:?}", e.0))
    }
}

impl From<reqwest::Error> for VmCliError {
    fn from(e: reqwest::Error) -> Self {
        let mut error_message = format!("{:?}", e);
        if error_message.contains("Invalid chunk size")
            || error_message.contains("missing size digit")
        {
            error_message = "请求的数据量过大，建议缩短时间步长或导出范围！".to_string();
            VmCliError::VlogVmCliError(error_message)
        } else {
            VmCliError::ReqwestError(e)
        }
    }
}
