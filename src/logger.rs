use log::LevelFilter;

use crate::error::SealedResult;

pub async fn init_logging(log_level: LevelFilter) -> SealedResult {
    env_logger::builder().filter_level(log_level).init();
    flexi_logger::init();

    // TODO: setup tracing

    Ok(())
}
