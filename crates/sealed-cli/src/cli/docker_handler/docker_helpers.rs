use std::{fmt::Display, path::PathBuf, process::Command};

use clap::{Args, Parser};
use serde::{Deserialize, Serialize};

#[derive(Args, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DockerBuilderOptions {
    #[arg(long)]
    pub builder_name: Option<String>,
    #[arg(long)]
    pub out_dir: Option<String>,
    #[arg(long)]
    pub print_dockerfile: bool,
    #[arg(long)]
    pub tags: Vec<String>,
    #[arg(long)]
    pub labels: Vec<String>,
    #[arg(long)]
    pub quiet: bool,
    #[arg(long)]
    pub no_cache: bool,
    #[arg(long)]
    pub platforms: Vec<String>,
    #[arg(long)]
    pub current_dir: Option<String>,
    #[arg(long)]
    pub cpu_quota: Option<String>,
    #[arg(long)]
    pub cpu_period: Option<String>,
    #[arg(long, default_value = "100")]
    pub cpu_share: Option<String>,
    #[arg(long, default_value = "8096000")]
    pub memory: Option<String>,
    #[arg(long, default_value = "16192000")]
    pub memory_swap: Option<String>,
    #[arg(long)]
    pub verbose: bool,
    #[arg(long)]
    pub docker_host: Option<String>,
    #[arg(long)]
    pub docker_tls_verify: Option<String>,
    #[arg(long)]
    pub docker_output: Option<String>,
    #[arg(long)]
    pub docker_cert_path: Option<String>,
    #[arg(long)]
    pub dockerfile: Option<String>,
    #[arg(long, short = 'a')]
    pub build_args: Vec<String>,
}

impl Default for DockerBuilderOptions {
    fn default() -> Self {
        Self {
            builder_name: None,
            out_dir: None,
            print_dockerfile: false,
            tags: vec!["latest".to_string()],
            labels: vec![],
            quiet: false,
            no_cache: false,
            platforms: vec![],
            current_dir: None,
            cpu_quota: Some("50000".to_string()),
            cpu_period: Some("100000".to_string()),
            cpu_share: None,
            memory: Some("8096000".to_string()),
            memory_swap: Some("16192000".to_string()),
            verbose: false,
            docker_host: None,
            docker_tls_verify: None,
            docker_output: None,
            docker_cert_path: None,
            dockerfile: None,
            build_args: vec![],
        }
    }
}
#[derive(Args, Debug, Clone, Serialize, Deserialize, Default)]
pub struct DockerSpecificArgs {
    /// Repository
    #[arg(long, short = 'r', alias = "repo")]
    pub repository: Option<String>,
    /// Branch
    #[arg(long, short = 'b')]
    pub branch: Option<String>,

    /// Image
    #[arg(long, short, alias = "img", conflicts_with = "repository")]
    pub image: Option<String>,

    /// Tag
    #[arg(long, short, default_value = "latest", conflicts_with = "branch")]
    pub tag: Option<String>,
}

#[derive(Parser, Debug, Clone, Serialize, Deserialize)]
pub struct DockerInstanceOption {
    #[arg(long, alias = "config")]
    pub config_file: Option<String>,
    /// Remove container when it exits
    #[arg(long, action)]
    pub rm: bool,
    /// Volumes
    #[arg(long, short = 'v', default_value = "default_volumes")]
    pub volumes: Vec<String>,
    /// Environment variables
    #[arg(long, short = 'e', alias = "e", default_value = "default_env")]
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

    #[arg(long, short = 's')]
    pub secrets: Option<Vec<String>>,

    #[command(flatten)]
    pub docker_config: DockerSpecificArgs,
}

impl Default for DockerInstanceOption {
    fn default() -> Self {
        Self {
            config_file: None,
            rm: true,
            volumes: vec![
                "/etc:/etc:ro".to_string(),
                "type=tmpfs,tmpfs-size=10000000000,tmpfs-mode=0777:/app,destination=/app"
                    .to_string(),
                "logging:/var/log:rw".to_string(),
            ],
            env: vec!["HOME=/app".to_string()],
            name: None,
            user: None,
            commands: vec![],
            docker_config: DockerSpecificArgs::default(),
            secrets: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DockerBind {
    pub config: String,
    pub mode: Option<String>,
}

impl Default for DockerBind {
    fn default() -> Self {
        DockerBind {
            config: "/etc:/etc".to_string(),
            mode: Some("ro".to_string()),
        }
    }
}

impl Display for DockerBind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let config = self.config.clone();
        let mode = self.mode.clone().unwrap_or("ro".to_string());
        write!(f, "{}:{}", config, mode)
    }
}

impl From<String> for DockerBind {
    fn from(bind: String) -> Self {
        let parts: Vec<&str> = bind.split(":").collect();
        if parts.len() != 2 {
            DockerBind {
                config: bind,
                mode: Some("ro".to_string()),
            }
        } else {
            DockerBind {
                config: parts[0].to_string(),
                mode: Some(parts[1].to_string()),
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

pub fn default_volumes() -> Vec<String> {
    vec![
        "/etc:/etc:ro".to_string(),
        "type=tmpfs,tmpfs-size=10000000000,tmpfs-mode=0777:/app,destination=/app".to_string(),
        "logging:/var/log:rw".to_string(),
    ]
}

pub fn default_env() -> Vec<String> {
    vec!["HOME=/app".to_string()]
}

pub fn command_to_string(command: &Command) -> String {
    let mut result = String::new();

    // Add the program
    result.push_str(command.get_program().to_str().unwrap());

    // Add the arguments
    for arg in command.get_args() {
        if let Some(arg_str) = arg.to_str() {
            result.push(' ');
            // Check if the argument needs to be quoted
            if arg_str.contains(char::is_whitespace) {
                result.push('"');
                result.push_str(arg_str.replace('"', "\\\"").as_str());
                result.push('"');
            } else {
                result.push_str(arg_str);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;

    use clap::{ArgMatches, FromArgMatches};

    use super::*;

    #[test]
    fn test_docker_image_optionsparsing() {
        let res = DockerInstanceOption::try_parse_from(vec![
            "--rm",
            "-v",
            "/etc:/etc:ro",
            "-v",
            "type=tmpfs,tmpfs-size=10000000000,tmpfs-mode=0777:/app,destination=/app",
            "-e",
            "HOME=/app",
            "-n",
            "bob",
        ]);
        assert!(res.is_ok());
        let docker_args = res.unwrap();
        assert_eq!(
            docker_args.volumes,
            vec![
                "/etc:/etc:ro",
                "type=tmpfs,tmpfs-size=10000000000,tmpfs-mode=0777:/app,destination=/app",
            ]
        );
        assert_eq!(docker_args.name, Some("bob".to_string()));
        assert_eq!(docker_args.env, vec!["HOME=/app"]);
    }

    #[test]
    fn test_docker_builder_options_parsing() {
        let opts = DockerBuilderOptions {
            builder_name: Some("test-builder".to_string()),
            out_dir: Some("/tmp".to_string()),
            print_dockerfile: true,
            tags: vec!["latest".to_string()],
            labels: vec!["maintainer=me".to_string()],
            quiet: false,
            no_cache: true,
            platforms: vec!["linux/amd64".to_string()],
            current_dir: Some("/tmp".to_string()),
            cpu_quota: Some("60000".to_string()),
            verbose: true,
            docker_host: Some("unix:///var/run/docker.sock".to_string()),
            docker_tls_verify: None,
            docker_output: Some("text".to_string()),
            docker_cert_path: None,
            cpu_period: Some("100000".to_string()),
            cpu_share: Some("100".to_string()),
            memory: Some("8096000".to_string()),
            memory_swap: Some("16192000".to_string()),
            dockerfile: None,
            build_args: vec![],
        };
        let serialized = serde_json::to_string(&opts).unwrap();
        let deserialized: DockerBuilderOptions = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.builder_name, Some("test-builder".to_string()));
        assert_eq!(deserialized.out_dir, Some("/tmp".to_string()));
    }
}
