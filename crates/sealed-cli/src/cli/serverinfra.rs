use clap::Parser;
use sealed_common::settings::{ServerArgs, Settings};
use sealed_server::Server;

use crate::error::SealedCliResult;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
#[command(arg_required_else_help = true)]
pub struct ServerInitArgs {
    #[command(subcommand)]
    pub subcommand: Subcommand,
}

#[derive(Parser, Debug, Clone)]
pub enum Subcommand {
    #[command(about = "Start the server")]
    Start(ServerStartArgs),
}

#[derive(Parser, Debug, Clone)]
pub struct ServerStartArgs {
    /// The port to run the server on
    #[arg(long, default_value_t = 9999)]
    port: u16,
}

impl From<ServerStartArgs> for ServerArgs {
    fn from(args: ServerStartArgs) -> Self {
        ServerArgs { port: args.port }
    }
}

pub async fn run(args: ServerInitArgs, _config: &Settings) -> SealedCliResult<()> {
    println!("Starting server infrastructure...");

    match args.subcommand {
        Subcommand::Start(args) => start_server(args.into()).await?,
    }

    Ok(())
}

async fn start_server(args: ServerArgs) -> SealedCliResult<()> {
    let server = Server::new(args).await;

    server.run().await?;

    Ok(())
}
