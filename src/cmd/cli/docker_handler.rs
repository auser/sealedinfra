use crate::{
    error::{SealedError, SealedResult},
    settings::Settings,
};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

mod build;
mod generate;
mod run;

pub(crate) mod docker_utils;

pub async fn run(args: DockerHandlerArgs, config: &Settings) -> SealedResult<()> {
    let docker_args = args.merge_with_config()?;

    match docker_args.subcmd {
        Some(SubCommand::Generate) => generate::run(docker_args, config).await,
        Some(SubCommand::Build) => build::run(docker_args, config).await,
        Some(SubCommand::Run) => run::run(docker_args, config).await,
        None => Err(SealedError::Runtime(anyhow::anyhow!(
            "No subcommand specified or unhandled command"
        ))),
    }
}

#[derive(Parser, Debug, Clone, Serialize, Deserialize, Default)]
pub struct DockerHandlerArgs {
    /// Config file path
    #[arg(long, short = 'f', alias = "config")]
    pub config_file: Option<PathBuf>,
    /// Remove container when it exits
    #[arg(long)]
    pub rm: bool,
    /// Bind mounts
    #[arg(long, short = 'B')]
    pub binds: Vec<String>,
    /// Volumes
    #[arg(long, short = 'v')]
    pub volumes: Vec<String>,
    /// Environment variables
    #[arg(long, short = 'e')]
    pub env: Vec<String>,
    /// Name
    #[arg(long, short = 'n')]
    pub name: Option<String>,
    /// User
    #[arg(long, short = 'u')]
    pub user: Option<String>,
    /// Commands
    #[arg(long, short = 'c')]
    pub commands: Vec<String>,

    #[arg(long, short = 'r', alias = "repo")]
    pub repository: Option<String>,
    #[arg(long, short = 'b')]
    pub branch: Option<String>,

    #[arg(long, short, alias = "img", conflicts_with = "repository")]
    pub image: Option<String>,
    #[arg(long, short, default_value = "latest", conflicts_with = "branch")]
    pub tag: Option<String>,

    #[command(subcommand)]
    #[serde(skip)]
    pub subcmd: Option<SubCommand>,
}

#[derive(Debug, Parser, Clone)]
pub enum SubCommand {
    /// Generate the docker run command
    Generate,
    /// Build the docker run command
    Build,
    /// Run the docker run command
    Run,
}

impl DockerHandlerArgs {
    fn merge_with_config(self) -> SealedResult<Self> {
        let mut merged = Self {
            binds: vec!["/etc:/etc:ro".to_string()],
            volumes: vec![
                "type=tmpfs,tmpfs-size=10000000000,tmpfs-mode=0777:/app,destination=/app"
                    .to_string(),
            ],
            env: vec!["HOME=/app".to_string()],
            ..self.clone()
        };

        if let Some(config_file) = &self.config_file {
            let config = std::fs::read_to_string(config_file).map_err(|e| {
                SealedError::Runtime(anyhow::anyhow!("Failed to read config file: {}", e))
            })?;
            let config: serde_yaml::Value = serde_yaml::from_str(&config).map_err(|e| {
                SealedError::Runtime(anyhow::anyhow!("Failed to parse config file: {}", e))
            })?;

            if let Some(repo) = config.get("repository") {
                merged.repository =
                    Some(repo.as_str().expect("Failed to get repository").to_string());
            }
            if let Some(branch) = config.get("branch") {
                merged.branch = Some(branch.as_str().expect("Failed to get branch").to_string());
            }
            if let Some(image) = config.get("image") {
                merged.image = Some(image.as_str().expect("Failed to get image").to_string());
            }
            if let Some(user) = config.get("user") {
                merged.user = Some(user.as_str().expect("Failed to get user").to_string());
            }
            if let Some(env) = config.get("env") {
                merged.env.extend(
                    env.as_sequence()
                        .expect("Failed to get env sequence")
                        .iter()
                        .map(|kv| {
                            kv.as_str()
                                .expect("Failed to get key-value pair")
                                .to_string()
                        }),
                );
            }
        }

        // Override with CLI args if provided
        if !self.binds.is_empty() {
            merged.binds = self.binds;
        }
        if !self.volumes.is_empty() {
            merged.volumes = self.volumes;
        }
        if !self.env.is_empty() {
            merged.env = self.env;
        }
        merged.repository = self.repository.or(merged.repository);
        merged.branch = self.branch.or(merged.branch);
        merged.image = self.image.or(merged.image);
        merged.tag = self.tag.or(merged.tag);
        merged.name = self.name.or(merged.name);
        merged.user = self.user.or(merged.user);
        if !self.commands.is_empty() {
            merged.commands = self.commands;
        }

        Ok(merged)
    }
}
