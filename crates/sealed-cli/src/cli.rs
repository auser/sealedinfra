use std::path::PathBuf;

use clap::Parser;
use info::InfoArgs;
use sealed_common::{metadata::LevelFilter, util::tracing::setup_tracing};

use crate::{error::SealedCliResult, init::init_config};

mod cluster;
mod docker_handler;
mod info;
pub(crate) mod sealedinfra;
mod serverinfra;
mod terraform;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
#[command(arg_required_else_help = true)]
pub struct Cli {
    #[arg(short, long)]
    pub verbose: bool,

    #[arg(short, long)]
    pub root: Option<PathBuf>,

    #[clap(short('l'), long, value_name("LEVEL"), default_value("info"))]
    pub log_level: LevelFilter,

    #[arg(short, long)]
    pub settings: Option<PathBuf>,

    #[command(subcommand)]
    pub cmd: Command,
}

impl Default for Cli {
    fn default() -> Self {
        Cli {
            settings: Some(PathBuf::from("config/config.yaml")),
            verbose: false,
            root: None,
            log_level: LevelFilter::INFO,
            cmd: Command::Info(InfoArgs {}),
        }
    }
}

#[derive(Debug, Parser, Clone)]
pub enum Command {
    #[command(about = "Show information about sealedinfra")]
    Info(InfoArgs),
    #[command(about = "Manage clusters", alias = "c")]
    Cluster(cluster::ClusterArgs),
    #[command(about = "Manage terraform", alias = "t")]
    Terraform(terraform::TerraformArgs),
    #[command(about = "Manage sealedinfra", alias = "sealedinfra")]
    SI(Box<sealedinfra::SealedInfraArgs>),
    #[command(about = "Handle docker generation", alias = "dh")]
    Docker(Box<docker_handler::DockerHandlerArgs>),
    #[command(about = "Manage server infrastructure")]
    Server(serverinfra::ServerInitArgs),
}

pub async fn exec() -> SealedCliResult {
    dotenv::dotenv().ok();
    let cli = Cli::parse();
    setup_tracing(Some(cli.log_level)).await;
    let cfg = init_config(&cli).expect("Unable to initialize config");

    match cli.cmd {
        Command::Info(args) => info::run(args, cfg).await?,
        Command::Cluster(args) => cluster::run(args, cfg).await?,
        Command::Terraform(args) => terraform::run(args, cfg).await?,
        Command::SI(args) => sealedinfra::run(*args, cfg).await?,
        Command::Docker(args) => docker_handler::run(*args, cfg).await?,
        // #[cfg(feature = "server")]
        Command::Server(args) => serverinfra::run(args, cfg).await?,
    }
    Ok(())
}
