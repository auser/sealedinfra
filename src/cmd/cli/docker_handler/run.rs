use crate::{error::SealedResult, settings::Settings};

use super::DockerHandlerArgs;

pub async fn run(_args: DockerHandlerArgs, _config: &Settings) -> SealedResult<()> {
    Ok(())
}
