use crate::{error::SealedResult, settings::Settings};

use super::DockerHandlerArgs;

pub async fn run(args: DockerHandlerArgs, _config: &Settings) -> SealedResult<()> {
    println!("Generating docker run command: {:#?}", args);

    Ok(())
}
