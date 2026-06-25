use crate::{error::CliError, proj_config::ReqwestConfig};

pub trait Runnable {
    fn run(&self, config: &ReqwestConfig) -> impl Future<Output = Result<(), CliError>> + Send;
}
