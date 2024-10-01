use std::path::{Path, PathBuf};

use clap::Parser;
use log::info;

use crate::{error::SealedResult, settings::Settings, util::command::stream_command_output};

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
#[command(arg_required_else_help = true)]
pub struct ClusterArgs {
    #[command(subcommand)]
    pub subcommand: Subcommand,
}

#[derive(Parser, Debug, Clone)]
pub enum Subcommand {
    #[command(about = "Create a new cluster")]
    Create(CreateArgs),
    #[command(about = "Delete a cluster")]
    Delete(DeleteArgs),
    #[command(about = "List all clusters")]
    List(ListArgs),
}

#[derive(Parser, Debug, Clone)]
pub struct CreateArgs {
    #[arg(short, long, default_value = "default")]
    pub name: String,

    #[arg(short, long)]
    pub kind_config: Option<PathBuf>,
}

#[derive(Parser, Debug, Clone)]
pub struct DeleteArgs {
    #[arg(short, long)]
    pub name: String,
}

#[derive(Parser, Debug, Clone)]
pub struct ListArgs {}

// Create a new cluster
async fn create(args: CreateArgs, _config: &Settings) -> SealedResult<()> {
    let cluster_name = args.name;
    info!("Creating cluster {}", cluster_name);

    let kind_config = args.kind_config.unwrap_or_else(get_default_kind_config);
    info!("Using kind config {}", kind_config.display());

    stream_command_output(
        "kind",
        &[
            "create",
            "cluster",
            "--name",
            &cluster_name,
            "--config",
            &kind_config.to_string_lossy(),
        ],
    )
    .await?;

    Ok(())
}

async fn delete(args: DeleteArgs, _config: &Settings) -> SealedResult<()> {
    let cluster_name = args.name;
    info!("Deleting cluster {}", cluster_name);

    stream_command_output("kind", &["delete", "cluster", "--name", &cluster_name]).await?;

    Ok(())
}

async fn list(_args: ListArgs, _config: &Settings) -> SealedResult<()> {
    stream_command_output("kind", &["get", "clusters"]).await?;

    Ok(())
}

pub async fn run(args: ClusterArgs, config: &Settings) -> SealedResult<()> {
    match args.subcommand {
        Subcommand::Create(args) => create(args, config).await,
        Subcommand::Delete(args) => delete(args, config).await,
        Subcommand::List(args) => list(args, config).await,
    }
}

fn get_default_kind_config() -> PathBuf {
    let path = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let path = Path::new(&path);
    path.join("config").join("kind-config.yaml")
}
