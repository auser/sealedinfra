use sealed_common::{info, settings::Settings};
use tokio::process::Command;

use crate::error::{SealedCliError, SealedCliResult};

use super::DockerHandlerArgs;

pub async fn run(args: &mut DockerHandlerArgs, config: &Settings) -> SealedCliResult<()> {
    let repo = args.with_repo(config)?;
    info!("Repository cloned: {}", repo.path().display());

    let cmd = args.to_docker_run_command_string(config)?;

    let mut command = Command::new("sh");
    command.arg("-c").arg(cmd);

    let env = args.get_env_prefix();

    // Apply environment variables
    for env_var in env.iter() {
        let parts: Vec<&str> = env_var.splitn(2, '=').collect();
        if parts.len() == 2 {
            command.env(parts[0], parts[1]);
        }
    }

    let output = command
        .output()
        .await
        .map_err(|e| SealedCliError::Runtime(e.to_string()))?;

    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    if output.status.success() {
        Ok(())
    } else {
        Err(SealedCliError::Runtime(
            "Docker build command failed".to_string(),
        ))
    }
}
