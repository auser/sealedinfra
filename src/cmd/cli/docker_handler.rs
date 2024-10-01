use crate::{
    error::{SealedError, SealedResult},
    settings::Settings,
};
use clap::Parser;

mod build;
mod run;

pub async fn run(args: DockerHandlerArgs, config: &Settings) -> SealedResult<()> {
    let docker_args = DockerHandlerArgs::builder()
        .repo_branch(args.repository.clone(), args.branch.clone())
        .image_tag(args.image.clone(), args.tag.clone())
        .binds(args.binds.clone())
        .volumes(args.volumes.clone())
        .env(args.env.clone())
        .name(args.name.clone())
        .commands(args.commands.clone())
        .build()
        .map_err(|e| SealedError::Runtime(anyhow::anyhow!(e)))?;

    match args.subcmd {
        Some(SubCommand::Build) => build::run(docker_args, config).await,
        Some(SubCommand::Run) => run::run(docker_args, config).await,
        None => Err(SealedError::Runtime(anyhow::anyhow!(
            "No subcommand specified or unhandled command"
        ))),
    }
}

#[derive(Parser, Debug, Clone)]
pub struct DockerHandlerArgs {
    #[arg(long, short = 'B')]
    pub binds: Vec<String>,
    #[arg(long, short = 'v')]
    pub volumes: Vec<String>,
    #[arg(long, short = 'e')]
    pub env: Vec<String>,
    #[arg(long, short = 'n')]
    pub name: Option<String>,
    #[arg(long, short = 'c')]
    pub commands: Vec<String>,

    #[arg(long, short = 'r', alias = "repo")]
    pub repository: Option<String>,
    #[arg(long, short = 'b', default_value = "main")]
    pub branch: Option<String>,

    #[arg(long, short, alias = "img", conflicts_with = "repository")]
    pub image: Option<String>,
    #[arg(long, short, default_value = "latest", conflicts_with = "branch")]
    pub tag: Option<String>,

    #[command(subcommand)]
    pub subcmd: Option<SubCommand>,
}

#[derive(Debug, Parser, Clone)]
pub enum SubCommand {
    Build,
    Run,
}

#[derive(Debug, Clone, Default)]
pub struct DockerHandlerArgsBuilder {
    pub binds: Vec<String>,
    pub volumes: Vec<String>,
    pub env: Vec<String>,
    pub name: Option<String>,
    pub commands: Vec<String>,
    pub repository: Option<String>,
    pub branch: Option<String>,
    pub image: Option<String>,
    pub tag: Option<String>,
}

impl DockerHandlerArgsBuilder {
    pub fn repo_branch(mut self, repository: Option<String>, branch: Option<String>) -> Self {
        self.repository = repository;
        self.branch = branch;
        self
    }

    pub fn image_tag(mut self, image: Option<String>, tag: Option<String>) -> Self {
        self.image = image;
        self.tag = tag;
        self
    }

    pub fn binds(mut self, binds: Vec<String>) -> Self {
        self.binds = binds;
        self
    }

    pub fn volumes(mut self, volumes: Vec<String>) -> Self {
        self.volumes = volumes;
        self
    }

    pub fn env(mut self, env: Vec<String>) -> Self {
        self.env = env;
        self
    }

    pub fn name(mut self, name: Option<String>) -> Self {
        self.name = name;
        self
    }

    pub fn commands(mut self, commands: Vec<String>) -> Self {
        self.commands = commands;
        self
    }

    pub fn build(self) -> Result<DockerHandlerArgs, &'static str> {
        if self.repository.is_some() && self.image.is_some() {
            return Err("Cannot specify both repo_branch and image_tag");
        }
        if self.repository.is_none() && self.image.is_none() {
            return Err("Must specify either repo_branch or image_tag");
        }

        Ok(DockerHandlerArgs {
            binds: self.binds,
            volumes: self.volumes,
            env: self.env,
            name: self.name,
            commands: self.commands,
            repository: self.repository,
            branch: self.branch,
            image: self.image,
            tag: self.tag,
            subcmd: None,
        })
    }
}

impl DockerHandlerArgs {
    pub fn builder() -> DockerHandlerArgsBuilder {
        DockerHandlerArgsBuilder::default()
    }
}
