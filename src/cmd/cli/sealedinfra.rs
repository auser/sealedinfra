use clap::Parser;

use crate::sealed::installer;
use crate::{error::SealedResult, settings::Settings};

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
#[command(arg_required_else_help = true)]
pub struct SealedInfraArgs {
    #[command(subcommand)]
    pub subcommand: Subcommand,

    #[command(flatten)]
    pub install: InstallArgs,
}

#[derive(Parser, Debug, Clone)]
pub enum Subcommand {
    #[command(about = "Create a new cluster")]
    Install(InstallArgs),
}

#[derive(Parser, Debug, Clone)]
pub struct InstallArgs {
    /// Run a cut down version of Bionic for integration testing
    #[arg(long, default_value_t = false)]
    testing: bool,
    /// Don't install the operator
    #[arg(long, default_value_t = false)]
    no_operator: bool,
    /// Install ingress
    #[arg(long, default_value_t = false)]
    no_ingress: bool,

    /// SealedInfra namespace
    #[arg(long, default_value = "fp")]
    pub namespace: String,

    /// Operator namespace
    #[arg(long, default_value = "fp-system")]
    pub operator_namespace: String,

    /// Setup for development?
    #[arg(long, default_value_t = false)]
    pub development: bool,

    /// Install pgAdmin?
    #[arg(long, default_value_t = false)]
    pgadmin: bool,
}

pub async fn run(args: SealedInfraArgs, config: &Settings) -> SealedResult<()> {
    match args.subcommand {
        Subcommand::Install(args) => {
            installer::install(args, config).await?;
        }
    }
    Ok(())
}
