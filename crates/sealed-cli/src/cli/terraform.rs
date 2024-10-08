#![allow(dead_code)]
use clap::Parser;
use sealed_common::{
    settings::Settings,
    terraform::{init_terraform, TerraformOptions},
};

use crate::error::SealedCliResult;

#[derive(Parser, Debug, Clone)]
pub struct TerraformArgs {
    #[arg(short, long)]
    pub dir: Option<String>,

    #[command(subcommand)]
    pub command: TerraformCommand,
}

#[derive(Parser, Debug, Clone)]
pub enum TerraformCommand {
    Init(InitArgs),
}

#[derive(Parser, Debug, Clone)]
pub struct InitArgs {
    #[arg(short, long)]
    pub dir: Option<String>,
}

pub async fn init(args: InitArgs, _config: &Settings) -> SealedCliResult<()> {
    let opts = TerraformOptions::new().with_dir(args.dir).clone().build();
    init_terraform(&opts).await?;
    Ok(())
}

pub async fn run(args: TerraformArgs, _config: &Settings) -> SealedCliResult<()> {
    println!("Terraform args: {:?}", args);
    eprintln!("Terraform not implemented yet");
    Ok(())
    // match args.command {
    //     TerraformCommand::Init(init_args) => init(init_args, config).await,
    // }
}
