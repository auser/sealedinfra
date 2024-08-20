use std::path::PathBuf;

use clap::Parser;
use info::InfoArgs;
use log::LevelFilter;

use crate::{error::SealedResult, logger::init_logging, settings::init_config};

mod cluster;
mod info;
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

    #[command(subcommand)]
    pub cmd: Command,
}

impl Default for Cli {
    fn default() -> Self {
        Cli {
            verbose: false,
            root: None,
            log_level: LevelFilter::Info,
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
}

pub async fn exec() -> SealedResult {
    let cli = Cli::parse();
    init_logging(cli.log_level).await?;
    let cfg = init_config(cli.root).expect("Unable to initialize config");

    match cli.cmd {
        Command::Info(args) => info::run(args, &cfg).await?,
        Command::Cluster(args) => cluster::run(args, &cfg).await?,
        Command::Terraform(args) => terraform::run(args, &cfg).await?,
    }
    Ok(())
}
