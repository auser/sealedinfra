#![allow(unused)]
use std::path::Path;

use crate::error::{SealedCliError, SealedCliResult};
use anyhow::Context;
use clap::{Args, Parser};
use docker_helpers::{DockerBuilderOptions, DockerInstanceOption};
use git2::Repository;
use log::{debug, info};
use sealed_common::{
    fs_utils::{expand_path, find_file_by_name},
    git_ops::parse_repo_name,
    settings::Settings,
};
use sealed_services::git_repo_service::GitRepoService;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::fs::canonicalize;
use tokio::process::Command;

mod build;
// mod generate;
mod docker_helpers;
mod run;

pub async fn run(args: DockerHandlerArgs, config: &Settings) -> SealedCliResult<()> {
    let mut docker_args = args.clone();
    let (mut docker_args, config) = docker_args.merge_with_config(config)?;
    docker_args.validate()?;

    match &docker_args.subcmd {
        // Some(SubCommand::Generate) => generate::run(docker_args, config).await,
        Some(SubCommand::Build) => build::run(docker_args, &config).await,
        Some(SubCommand::Run) => run::run(docker_args, &config).await,
        Some(_cmd) => Err(SealedCliError::Runtime(
            "Unhandled command: for now".to_string(),
        )),
        None => Err(SealedCliError::Runtime(
            "No subcommand specified or unhandled command".to_string(),
        )),
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
    pub fn build_command(&self, config: &Settings) -> SealedCliResult<Command> {
        let command = self.to_docker_buildx_command_string(config)?;
        let env_prefix = self.get_env_prefix();
        let mut cmd = Command::new("sh");

        for env_var in env_prefix.iter() {
            let parts: Vec<&str> = env_var.splitn(2, '=').collect();
            if parts.len() == 2 {
                cmd.env(parts[0], parts[1]);
            }
        }
        if let Some(ref current_dir) = self.docker.builder.current_dir {
            cmd.current_dir(current_dir);
        }
        cmd.arg("-c").arg(command);
        Ok(cmd)
    }

    pub fn run_command(&self, config: &Settings) -> SealedCliResult<Command> {
        let command = self.to_docker_run_command_string(config)?;
        let env_prefix = self.get_env_prefix();
        let mut cmd = Command::new("sh");
        for env_var in env_prefix.iter() {
            let parts: Vec<&str> = env_var.splitn(2, '=').collect();
            if parts.len() == 2 {
                cmd.env(parts[0], parts[1]);
            }
        }
        if let Some(ref repository) = self.docker.instance.docker_config.repository {
            cmd.current_dir(repository);
        } else if let Some(ref current_dir) = self.docker.builder.current_dir {
            cmd.current_dir(current_dir);
        }
        cmd.arg("-c").arg(command);
        Ok(cmd)
    }
}

impl DockerHandlerArgs {
    pub fn to_docker_buildx_command_string(&self, config: &Settings) -> SealedCliResult<String> {
        let repo_name = self.get_repo_name()?;
        let mut cmd_parts: Vec<String> = vec![
            "docker".to_string(),
            "buildx".to_string(),
            "build".to_string(),
        ];

        // Add the build context (current directory)
        let in_dir = self.docker.builder.current_dir.as_deref().unwrap_or(".");

        // Add builder options
        if let Some(ref builder_name) = self.docker.builder.builder_name {
            cmd_parts.extend_from_slice(&["--builder".to_string(), builder_name.to_string()]);
        }
        if let Some(ref out_dir) = self.docker.builder.out_dir {
            cmd_parts.extend_from_slice(&["--output".to_string(), out_dir.to_string()]);
        }
        if self.docker.builder.print_dockerfile {
            cmd_parts.push("--print".to_string());
        }
        for tag in &self.docker.builder.tags {
            cmd_parts.extend_from_slice(&["-t".to_string(), tag.to_string()]);
        }
        for label in &self.docker.builder.labels {
            cmd_parts.extend_from_slice(&["--label".to_string(), label.to_string()]);
        }
        if self.docker.builder.quiet {
            cmd_parts.push("--quiet".to_string());
        }
        if self.docker.builder.no_cache {
            cmd_parts.push("--no-cache".to_string());
        }
        for platform in &self.docker.builder.platforms {
            cmd_parts.extend_from_slice(&["--platform".to_string(), platform.to_string()]);
        }
        if let Some(ref cpu_quota) = self.docker.builder.cpu_quota {
            cmd_parts.extend_from_slice(&["--cpu-quota".to_string(), cpu_quota.to_string()]);
        }
        if let Some(ref cpu_period) = self.docker.builder.cpu_period {
            cmd_parts.extend_from_slice(&["--cpu-period".to_string(), cpu_period.to_string()]);
        }
        if let Some(ref cpu_share) = self.docker.builder.cpu_share {
            cmd_parts.extend_from_slice(&["--cpu-shares".to_string(), cpu_share.to_string()]);
        }
        if let Some(ref memory) = self.docker.builder.memory {
            cmd_parts.extend_from_slice(&["--memory".to_string(), memory.to_string()]);
        }
        if let Some(ref memory_swap) = self.docker.builder.memory_swap {
            cmd_parts.extend_from_slice(&["--memory-swap".to_string(), memory_swap.to_string()]);
        }
        if let Some(ref dockerfile) = self.docker.builder.dockerfile {
            cmd_parts.extend_from_slice(&["--file".to_string(), dockerfile.to_string()]);
        }
        if self.docker.builder.verbose {
            cmd_parts.push("--verbose".to_string());
        }

        for arg in &self.docker.builder.build_args {
            cmd_parts.extend_from_slice(&["--build-arg".to_string(), arg.to_string()]);
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
        cmd_parts.extend_from_slice(&["-t".to_string(), tag.to_string()]);

        match &self.docker.builder.dockerfile {
            Some(dockerfile) => {
                cmd_parts.extend_from_slice(&["-f".to_string(), dockerfile.to_string()]);
            }
            None => {
                if let Ok(found_dockerfile) = find_file_by_name(Path::new(in_dir), "Dockerfile") {
                    if let Some(path_str) = found_dockerfile.to_str() {
                        let dockerfile_path = path_str.to_owned();
                        let dockerfile_path = expand_path(Path::new(&dockerfile_path));
                        cmd_parts.extend_from_slice(&[
                            "-f".to_string(),
                            format!("{}", dockerfile_path.to_string_lossy()),
                        ]);
                    }
                }
            }
        }

        if let Some(ref secrets) = self.docker.instance.secrets {
            for secret in secrets {
                cmd_parts.extend_from_slice(&["--secret".to_string(), secret.to_string()]);
            }
        }
        if let Some(ref host_key) = config.ssh_key {
            // --secret id=ssh_priv_key,src=$HOME/.ssh/herring_id_ed25519
            let host_key = expand_path(host_key.as_path());
            cmd_parts.extend_from_slice(&[
                "--secret".to_string(),
                format!("id=ssh_priv_key,src={}", host_key.display()),
            ]);
        }

        let mut env_prefix: Vec<String> = Vec::new();

        if let Some(ref host) = self.docker.builder.docker_host {
            env_prefix.push(format!("DOCKER_HOST={}", shell_escape::escape(host.into())));
        }

        if let Some(ref tls_verify) = self.docker.builder.docker_tls_verify {
            env_prefix.push(format!(
                "DOCKER_TLS_VERIFY={}",
                shell_escape::escape(tls_verify.into())
            ));
        }

        if let Some(ref cert_path) = self.docker.builder.docker_cert_path {
            env_prefix.push(format!(
                "DOCKER_CERT_PATH={}",
                shell_escape::escape(cert_path.into())
            ));
        }

        cmd_parts.push(in_dir.to_string());

        let cmd_string = cmd_parts
            .into_iter()
            .map(|s| shell_escape::escape(s.into()))
            .collect::<Vec<_>>()
            .join(" ");

        Ok(cmd_string)
    }

    pub fn to_docker_run_command_string(&self, config: &Settings) -> SealedCliResult<String> {
        let repo_name = self.get_repo_name()?;
        let mut cmd_parts = vec!["docker".to_string(), "run".to_string()];

        if self.docker.instance.rm {
            cmd_parts.push("--rm".to_string());
        }

        for volume in &self.docker.instance.volumes {
            cmd_parts.extend_from_slice(&["-v".to_string(), volume.to_string()]);
        }

        for env_var in &self.docker.instance.env {
            cmd_parts.extend_from_slice(&["-e".to_string(), env_var.to_string()]);
        }

        if let Some(ref name) = self.docker.instance.name {
            cmd_parts.extend_from_slice(&["--name".to_string(), name.to_string()]);
        }

        if let Some(ref user) = self.docker.instance.user {
            cmd_parts.extend_from_slice(&["-u".to_string(), user.to_string()]);
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
        cmd_parts.push(tag.to_string());

        cmd_parts.extend(self.docker.instance.commands.iter().map(|s| s.to_string()));

        let cmd_string = cmd_parts
            .into_iter()
            .map(|s| shell_escape::escape(s.into()))
            .collect::<Vec<_>>()
            .join(" ");

        Ok(cmd_string)
    }

    pub fn get_env_prefix(&self) -> Vec<String> {
        let mut env_prefix: Vec<String> = Vec::new();

        if let Some(ref host) = self.docker.builder.docker_host {
            env_prefix.push(format!("DOCKER_HOST={}", shell_escape::escape(host.into())));
        }

        if let Some(ref tls_verify) = self.docker.builder.docker_tls_verify {
            env_prefix.push(format!(
                "DOCKER_TLS_VERIFY={}",
                shell_escape::escape(tls_verify.into())
            ));
        }

        if let Some(ref cert_path) = self.docker.builder.docker_cert_path {
            env_prefix.push(format!(
                "DOCKER_CERT_PATH={}",
                shell_escape::escape(cert_path.into())
            ));
        }
        env_prefix
    }
}

impl DockerHandlerArgs {
    pub fn validate(&mut self) -> Result<(), SealedCliError> {
        let repo = self.docker.instance.docker_config.repository.clone();
        let branch = self.docker.instance.docker_config.branch.clone();
        let tag = self.docker.instance.docker_config.tag.clone();
        let image = self.docker.instance.docker_config.image.clone();

        if repo.is_none() && image.is_none() && image.is_none() && repo.is_none() {
            return Err(SealedCliError::Runtime(
                "No repository or image specified".to_string(),
            ));
        }

        if self.docker.instance.docker_config.repository.is_some() {
            let repo_as_path = self
                .docker
                .instance
                .docker_config
                .repository
                .clone()
                .unwrap();
            if Path::new(&repo_as_path).exists() {
                debug!("Repository path: {}", repo_as_path.clone());
                self.docker.builder.current_dir = Some(repo_as_path);
            }
        }
        Ok(())
    }

    pub fn get_repo_name(&self) -> SealedCliResult<String> {
        let repo = self.docker.instance.docker_config.repository.clone();
        let branch = self.docker.instance.docker_config.branch.clone();
        let tag = self.docker.instance.docker_config.tag.clone();
        let image = self.docker.instance.docker_config.image.clone();
        if let Some(repo) = repo {
            parse_repo_name(&repo).map_err(SealedCliError::from)
        } else if let Some(image) = image {
            Ok(image)
        } else {
            panic!("No repository or image specified");
        }
    }
    pub fn merge_with_config(
        &mut self,
        config: &Settings,
    ) -> SealedCliResult<(&mut Self, Settings)> {
        let mut config = config.clone();
        if let Some(config_file) = &self.docker.instance.config_file {
            let config_str =
                std::fs::read_to_string(config_file).context("Failed to read config file")?;
            let mut cfg: Value =
                serde_yaml::from_str(&config_str).context("Failed to parse config file")?;

            config = merge_config(config, &cfg);

            self.docker.instance = merge_instance(self.docker.instance.clone(), &cfg);
            self.docker.builder = merge_builder(self.docker.builder.clone(), &cfg);
        }

        Ok((self, config))
    }

    pub fn with_repo(&mut self, config: &Settings) -> SealedCliResult<Repository> {
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

fn merge_config(mut config: Settings, other: &Value) -> Settings {
    if let Some(ssh_key) = other.get("ssh_key") {
        config.ssh_key = Some(ssh_key.as_str().unwrap().to_string().into());
    }
    if let Some(working_directory) = other.get("working_directory") {
        config.working_directory = working_directory.as_str().unwrap().to_string().into();
    }
    config
}
