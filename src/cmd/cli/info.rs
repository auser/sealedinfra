use clap::Parser;

use crate::{error::SealedResult, settings::Settings};

#[derive(Parser, Debug, Clone)]
pub struct InfoArgs {}

pub async fn run(_args: InfoArgs, _config: &Settings) -> SealedResult<()> {
    println!(
        "{} {} ({})",
        std::env::var("CARGO_PKG_VERSION").unwrap(),
        std::env::var("VERGEN_BUILD_DATE").unwrap(),
        std::env::var("VERGEN_GIT_SHA").unwrap()[..8].to_string()
    );
    Ok(())
}
