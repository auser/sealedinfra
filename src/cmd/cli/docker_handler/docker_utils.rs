use std::{fmt::Display, path::PathBuf};

use tokio::process::Command;

use crate::{error::SealedResult, settings::Settings, util::git_ops::parse_repo_name};

use super::DockerHandlerArgs;

pub fn build_docker_run_command(
    args: DockerHandlerArgs,
    repo_root: &PathBuf,
    _settings: &Settings,
) -> SealedResult<Command> {
    let mut command: Command = args.into();
    command.current_dir(repo_root);
    Ok(command)
}

impl From<DockerHandlerArgs> for Command {
    fn from(args: DockerHandlerArgs) -> Self {
        let mut command = Command::new("docker");
        command.arg("run");
        if args.rm {
            command.arg("--rm");
        }
        for bind in args.binds {
            let bind = DockerBind::from(bind);
            command.arg("-v");
            command.arg(format!(
                "{}:/{}:{}",
                bind.host_path, bind.container_path, bind.mode
            ));
        }
        for volume in args.volumes {
            let volume = DockerBind::from(volume);
            command.arg("-v");
            command.arg(format!(
                "{}:/{}:{}",
                volume.host_path, volume.container_path, volume.mode
            ));
        }
        for env in args.env {
            let env = DockerEnv::from(env);
            command.arg("-e");
            command.arg(format!("{}={}", env.key, env.value));
        }
        if let Some(name) = args.name {
            command.arg("--name");
            command.arg(name);
        }
        if let Some(user) = args.user {
            command.arg("--user");
            command.arg(user);
        }
        command.arg(format!(
            "{}:{}",
            args.image
                .unwrap_or(parse_repo_name(&args.repository.unwrap()).unwrap()),
            args.tag.unwrap_or("latest".to_string())
        ));

        for cmd in args.commands {
            command.arg(cmd);
        }

        command
    }
}

#[derive(Debug, Clone)]
pub struct DockerBind {
    pub host_path: String,
    pub container_path: String,
    pub mode: String,
}

impl Default for DockerBind {
    fn default() -> Self {
        DockerBind {
            host_path: "/etc".to_string(),
            container_path: "/etc".to_string(),
            mode: "ro".to_string(),
        }
    }
}

impl Display for DockerBind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}",
            self.host_path, self.container_path, self.mode
        )
    }
}

impl From<String> for DockerBind {
    fn from(bind: String) -> Self {
        let parts: Vec<&str> = bind.split(":").collect();
        if parts.len() != 3 {
            DockerBind {
                host_path: parts[0].to_string(),
                container_path: parts[1].to_string(),
                mode: "ro".to_string(),
            }
        } else {
            DockerBind {
                host_path: parts[0].to_string(),
                container_path: parts[1].to_string(),
                mode: parts[2].to_string(),
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct DockerEnv {
    pub key: String,
    pub value: String,
}

impl Display for DockerEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}={}", self.key, self.value)
    }
}

impl From<String> for DockerEnv {
    fn from(env: String) -> Self {
        let parts: Vec<&str> = env.split("=").collect();
        DockerEnv {
            key: parts[0].to_string(),
            value: parts[1].to_string(),
        }
    }
}
