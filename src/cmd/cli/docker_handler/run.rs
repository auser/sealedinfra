use crate::{
    cmd::cli::docker_handler::{build::GitRepoService, docker_utils::build_docker_run_command},
    error::{SealedError, SealedResult},
    settings::Settings,
};

use super::DockerHandlerArgs;

pub async fn run(args: DockerHandlerArgs, config: &Settings) -> SealedResult<()> {
    let repo = args.clone().repository;
    if repo.is_none() {
        return Err(SealedError::Runtime(anyhow::anyhow!(
            "Repository is not set"
        )));
    }
    let branch = args.branch.clone().unwrap_or("main".to_string());
    let repo = GitRepoService::fetch(&repo.clone().unwrap(), &branch, config)?;

    tracing::info!("Repository cloned: {}", repo.path().display());

    let mut command = build_docker_run_command(args, &repo.path().to_path_buf(), config)?;
    let output = command.output().await?;
    println!("{:?}", output);
    Ok(())
}
