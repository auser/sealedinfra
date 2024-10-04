use crate::{
    cmd::cli::docker_handler::docker_utils::build_docker_build_command,
    error::{SealedError, SealedResult},
    settings::Settings,
};
use tokio::process::Command;

use super::DockerHandlerArgs;

pub async fn run(args: DockerHandlerArgs, config: &Settings) -> SealedResult<()> {
    let mut args = args.merge_with_config()?;
    let repo = args.with_repo(config)?;
    tracing::info!("Repository cloned: {}", repo.path().display());

    let (cmd, env) = args.to_docker_run_command_string()?;

    let mut command = Command::new("sh");
    command.arg("-c").arg(cmd);

    // Apply environment variables
    for env_var in env.split_whitespace() {
        let parts: Vec<&str> = env_var.splitn(2, '=').collect();
        if parts.len() == 2 {
            command.env(parts[0], parts[1]);
        }
    }

    let output = command
        .output()
        .await
        .map_err(|e| SealedError::Runtime(e.into()))?;

    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    if output.status.success() {
        Ok(())
    } else {
        Err(SealedError::Runtime(anyhow::anyhow!(
            "Docker build command failed"
        )))
    }
}
