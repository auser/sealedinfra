#![allow(unused)]
use crate::{
    error::{SealedError, SealedResult},
    services::git_repo_service::GitRepoService,
    settings::Settings,
    util::{
        docker_helpers::{default_volumes, DockerBuilderOptions, DockerInstanceOption},
        git_ops::parse_repo_name,
    },
};
use anyhow::Context;
use clap::{Args, Parser};
use git2::Repository;
use log::info;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use tokio::process::Command;

mod build;
// mod generate;
mod run;

pub(crate) mod docker_utils;

pub async fn run(args: DockerHandlerArgs, config: &Settings) -> SealedResult<()> {
    let docker_args = args.merge_with_config()?;
    docker_args.validate()?;

    match docker_args.subcmd {
        // Some(SubCommand::Generate) => generate::run(docker_args, config).await,
        Some(SubCommand::Build) => build::run(docker_args, config).await,
        Some(SubCommand::Run) => run::run(docker_args, config).await,
        Some(_cmd) => Err(SealedError::Runtime(anyhow::anyhow!(
            "Unhandled command: for now",
        ))),
        None => Err(SealedError::Runtime(anyhow::anyhow!(
            "No subcommand specified or unhandled command"
        ))),
    }
}

#[derive(Debug, Parser, Serialize, Deserialize, Default, Clone)]
pub struct DockerHandlerArgs {
    #[arg(long, short)]
    pub dry_run: bool,

    #[command(flatten)]
    pub docker: DockerCommandArgs,

    #[command(subcommand)]
    #[serde(skip)]
    pub subcmd: Option<SubCommand>,
}

#[derive(Args, Debug, Clone, Serialize, Deserialize, Default)]
pub struct DockerCommandArgs {
    #[command(flatten)]
    pub builder: DockerBuilderOptions,

    #[command(flatten)]
    pub instance: DockerInstanceOption,
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
    pub fn build_command(&self) -> SealedResult<Command> {
        let (command, _) = self.to_docker_buildx_command_string()?;
        let mut cmd = Command::new("sh");
        if let Some(ref current_dir) = self.docker.builder.current_dir {
            cmd.current_dir(current_dir);
        }
        cmd.arg("-c").arg(command);
        Ok(cmd)
    }

    pub fn run_command(&self) -> SealedResult<Command> {
        let (command, _) = self.to_docker_run_command_string()?;
        let mut cmd = Command::new("sh");
        if let Some(ref current_dir) = self.docker.builder.current_dir {
            cmd.current_dir(current_dir);
        }
        cmd.arg("-c").arg(command);
        Ok(cmd)
    }
}

impl DockerHandlerArgs {
    pub fn to_docker_buildx_command_string(&self) -> SealedResult<(String, String)> {
        let repo_name = self.get_repo_name()?;
        let mut cmd_parts = vec!["docker", "buildx", "build"];

        // Add builder options
        if let Some(ref builder_name) = self.docker.builder.builder_name {
            cmd_parts.extend_from_slice(&["--builder", builder_name]);
        }
        if let Some(ref out_dir) = self.docker.builder.out_dir {
            cmd_parts.extend_from_slice(&["--output", out_dir]);
        }
        if self.docker.builder.print_dockerfile {
            cmd_parts.push("--print");
        }
        for tag in &self.docker.builder.tags {
            cmd_parts.extend_from_slice(&["-t", tag]);
        }
        for label in &self.docker.builder.labels {
            cmd_parts.extend_from_slice(&["--label", label]);
        }
        if self.docker.builder.quiet {
            cmd_parts.push("--quiet");
        }
        if self.docker.builder.no_cache {
            cmd_parts.push("--no-cache");
        }
        for platform in &self.docker.builder.platforms {
            cmd_parts.extend_from_slice(&["--platform", platform]);
        }
        if let Some(ref cpu_quota) = self.docker.builder.cpu_quota {
            cmd_parts.extend_from_slice(&["--cpu-quota", cpu_quota]);
        }
        if let Some(ref cpu_period) = self.docker.builder.cpu_period {
            cmd_parts.extend_from_slice(&["--cpu-period", cpu_period]);
        }
        if let Some(ref cpu_share) = self.docker.builder.cpu_share {
            cmd_parts.extend_from_slice(&["--cpu-shares", cpu_share]);
        }
        if let Some(ref memory) = self.docker.builder.memory {
            cmd_parts.extend_from_slice(&["--memory", memory]);
        }
        if let Some(ref memory_swap) = self.docker.builder.memory_swap {
            cmd_parts.extend_from_slice(&["--memory-swap", memory_swap]);
        }
        if let Some(ref dockerfile) = self.docker.builder.dockerfile {
            cmd_parts.extend_from_slice(&["--file", dockerfile]);
        }
        if self.docker.builder.verbose {
            cmd_parts.push("--verbose");
        }

        for arg in &self.docker.builder.build_args {
            cmd_parts.extend_from_slice(&["--build-arg", arg]);
        }

        let tag = format!(
            "{}:{}",
            repo_name,
            self.docker
                .instance
                .docker_config
                .tag
                .clone()
                .unwrap_or_else(|| "latest".to_string())
        );
        cmd_parts.extend_from_slice(&["-t", &tag]);

        // Add the build context (current directory)
        let in_dir = self.docker.builder.current_dir.as_deref().unwrap_or(".");
        cmd_parts.push(in_dir);

        let mut env_prefix = String::new();
        if let Some(ref host) = self.docker.builder.docker_host {
            env_prefix.push_str(&format!(
                "DOCKER_HOST={} ",
                shell_escape::escape(host.into())
            ));
        }
        if let Some(ref tls_verify) = self.docker.builder.docker_tls_verify {
            env_prefix.push_str(&format!(
                "DOCKER_TLS_VERIFY={} ",
                shell_escape::escape(tls_verify.into())
            ));
        }
        if let Some(ref cert_path) = self.docker.builder.docker_cert_path {
            env_prefix.push_str(&format!(
                "DOCKER_CERT_PATH={} ",
                shell_escape::escape(cert_path.into())
            ));
        }

        let cmd_string = cmd_parts
            .into_iter()
            .map(|s| shell_escape::escape(s.into()))
            .collect::<Vec<_>>()
            .join(" ");

        Ok((cmd_string, env_prefix))
    }

    pub fn to_docker_run_command_string(&self) -> SealedResult<(String, String)> {
        let repo_name = self.get_repo_name()?;
        let mut cmd_parts = vec!["docker", "run"];

        if self.docker.instance.rm {
            cmd_parts.push("--rm");
        }

        for volume in &self.docker.instance.volumes {
            cmd_parts.extend_from_slice(&["-v", volume]);
        }

        for env_var in &self.docker.instance.env {
            cmd_parts.extend_from_slice(&["-e", env_var]);
        }

        if let Some(ref name) = self.docker.instance.name {
            cmd_parts.extend_from_slice(&["--name", name]);
        }

        if let Some(ref user) = self.docker.instance.user {
            cmd_parts.extend_from_slice(&["-u", user]);
        }

        let tag = format!(
            "{}:{}",
            repo_name,
            self.docker
                .instance
                .docker_config
                .tag
                .clone()
                .unwrap_or_else(|| "latest".to_string())
        );
        cmd_parts.push(&tag);

        cmd_parts.extend(self.docker.instance.commands.iter().map(|s| s.as_str()));

        let mut env_prefix = String::new();
        if let Some(ref host) = self.docker.builder.docker_host {
            env_prefix.push_str(&format!(
                "DOCKER_HOST={} ",
                shell_escape::escape(host.into())
            ));
        }
        if let Some(ref tls_verify) = self.docker.builder.docker_tls_verify {
            env_prefix.push_str(&format!(
                "DOCKER_TLS_VERIFY={} ",
                shell_escape::escape(tls_verify.into())
            ));
        }
        if let Some(ref cert_path) = self.docker.builder.docker_cert_path {
            env_prefix.push_str(&format!(
                "DOCKER_CERT_PATH={} ",
                shell_escape::escape(cert_path.into())
            ));
        }

        let cmd_string = cmd_parts
            .into_iter()
            .map(|s| shell_escape::escape(s.into()))
            .collect::<Vec<_>>()
            .join(" ");

        Ok((cmd_string, env_prefix))
    }
}

impl DockerHandlerArgs {
    pub fn validate(&self) -> Result<(), SealedError> {
        let repo = self.docker.instance.docker_config.repository.clone();
        let branch = self.docker.instance.docker_config.branch.clone();
        let tag = self.docker.instance.docker_config.tag.clone();
        let image = self.docker.instance.docker_config.image.clone();

        if repo.is_none() && image.is_none() && image.is_none() && repo.is_none() {
            return Err(SealedError::Runtime(anyhow::anyhow!(
                "No repository or image specified"
            )));
        }
        Ok(())
    }

    pub fn get_repo_name(&self) -> SealedResult<String> {
        let repo = self.docker.instance.docker_config.repository.clone();
        let branch = self.docker.instance.docker_config.branch.clone();
        let tag = self.docker.instance.docker_config.tag.clone();
        let image = self.docker.instance.docker_config.image.clone();
        if repo.is_some() {
            parse_repo_name(&repo.clone().unwrap())
        } else if image.is_some() {
            Ok(image.unwrap())
        } else {
            panic!("No repository or image specified");
        }
    }
    pub fn merge_with_config(mut self) -> anyhow::Result<Self> {
        if let Some(config_file) = &self.docker.instance.config_file {
            let config =
                std::fs::read_to_string(config_file).context("Failed to read config file")?;
            let config: Value =
                serde_yaml::from_str(&config).context("Failed to parse config file")?;

            self.docker.instance = merge_instance(self.docker.instance, &config);
            self.docker.builder = merge_builder(self.docker.builder, &config);
        }

        Ok(self)
    }

    pub fn with_repo(&mut self, config: &Settings) -> SealedResult<Repository> {
        let branch = self
            .docker
            .instance
            .docker_config
            .branch
            .clone()
            .unwrap_or("main".to_string());
        let repository = self
            .docker
            .instance
            .docker_config
            .repository
            .clone()
            .unwrap();
        let repo = GitRepoService::fetch(&repository, &branch, config)?;
        let short_sha = GitRepoService::short_sha(&repo)?;
        self.docker.instance.docker_config.tag = Some(short_sha);

        info!("Repository cloned: {}", repo.path().display());
        self.docker.builder.current_dir = Some(
            repo.path()
                .to_path_buf()
                .parent()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
        );
        Ok(repo)
    }
}

fn merge_instance(mut instance: DockerInstanceOption, config: &Value) -> DockerInstanceOption {
    if let Some(config) = config.as_mapping() {
        instance.name = get_str_value(config, "name").or(instance.name);
        instance.user = get_str_value(config, "user").or(instance.user);
        instance.commands = get_str_sequence(config, "commands").unwrap_or(instance.commands);
        instance.volumes = get_str_sequence(config, "volumes").unwrap_or(instance.volumes);
        instance.env = get_str_sequence(config, "env").unwrap_or(instance.env);
        instance.rm = get_bool_value(config, "rm").unwrap_or(instance.rm);

        if let Some(docker_config) = config.get("docker_config") {
            if let Some(docker_config) = docker_config.as_mapping() {
                instance.docker_config.repository = get_str_value(docker_config, "repository")
                    .or(instance.docker_config.repository);
                instance.docker_config.branch =
                    get_str_value(docker_config, "branch").or(instance.docker_config.branch);
                instance.docker_config.image =
                    get_str_value(docker_config, "image").or(instance.docker_config.image);
                instance.docker_config.tag =
                    get_str_value(docker_config, "tag").or(instance.docker_config.tag);
            }
        }
    }
    instance
}

fn merge_builder(mut builder: DockerBuilderOptions, config: &Value) -> DockerBuilderOptions {
    if let Some(config) = config.as_mapping() {
        builder.builder_name = get_str_value(config, "builder_name").or(builder.builder_name);
        builder.out_dir = get_str_value(config, "out_dir").or(builder.out_dir);
        builder.print_dockerfile =
            get_bool_value(config, "print_dockerfile").unwrap_or(builder.print_dockerfile);
        builder.tags = get_str_sequence(config, "tags").unwrap_or(builder.tags);
        builder.labels = get_str_sequence(config, "labels").unwrap_or(builder.labels);
        builder.quiet = get_bool_value(config, "quiet").unwrap_or(builder.quiet);
        builder.no_cache = get_bool_value(config, "no_cache").unwrap_or(builder.no_cache);
        builder.platforms = get_str_sequence(config, "platforms").unwrap_or(builder.platforms);
        builder.current_dir = get_str_value(config, "current_dir").or(builder.current_dir);
        builder.cpu_quota = get_str_value(config, "cpu_quota").or(builder.cpu_quota);
        builder.cpu_period = get_str_value(config, "cpu_period").or(builder.cpu_period);
        builder.cpu_share = get_str_value(config, "cpu_share").or(builder.cpu_share);
        builder.memory = get_str_value(config, "memory").or(builder.memory);
        builder.memory_swap = get_str_value(config, "memory_swap").or(builder.memory_swap);
        builder.verbose = get_bool_value(config, "verbose").unwrap_or(builder.verbose);
        builder.docker_host = get_str_value(config, "docker_host").or(builder.docker_host);
        builder.docker_tls_verify =
            get_str_value(config, "docker_tls_verify").or(builder.docker_tls_verify);
        builder.docker_output = get_str_value(config, "docker_output").or(builder.docker_output);
        builder.docker_cert_path =
            get_str_value(config, "docker_cert_path").or(builder.docker_cert_path);
    }
    builder
}

fn get_str_value(config: &serde_yaml::Mapping, key: &str) -> Option<String> {
    config.get(key).and_then(|v| v.as_str().map(String::from))
}

fn get_str_sequence(config: &serde_yaml::Mapping, key: &str) -> Option<Vec<String>> {
    config.get(key).and_then(|v| {
        v.as_sequence().map(|seq| {
            seq.iter()
                .filter_map(|item| item.as_str().map(String::from))
                .collect()
        })
    })
}

fn get_bool_value(config: &serde_yaml::Mapping, key: &str) -> Option<bool> {
    config.get(key).and_then(|v| v.as_bool())
}
