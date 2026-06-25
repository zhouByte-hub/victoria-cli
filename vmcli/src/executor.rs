use crate::{error::VmCliError, proj_config::ReqwestConfig};

pub trait Runnable {
    fn run(&self, config: &ReqwestConfig) -> impl Future<Output = Result<(), VmCliError>> + Send;
}
