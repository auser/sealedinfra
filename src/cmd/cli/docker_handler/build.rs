#![allow(unused)]
use anyhow::Context;
use git2::{
    build::{CheckoutBuilder, RepoBuilder},
    BranchType, Cred, ErrorCode, FetchOptions, RemoteCallbacks, Repository,
};
use log::debug;
use resolve_path::PathResolveExt;
use std::path::{Path, PathBuf};
use tokio::process::Command;
use tracing::info;

use crate::{
    cmd::cli::docker_handler::docker_utils::build_docker_build_command,
    error::{SealedError, SealedResult},
    services::git_repo_service::GitRepoService,
    settings::Settings,
    util::{docker_helpers::command_to_string, fs_utils::make_dirs, git_ops::parse_repo_name},
};

use super::DockerHandlerArgs;

pub async fn run(args: DockerHandlerArgs, config: &Settings) -> SealedResult<()> {
    let mut args = args.merge_with_config()?;
    args.validate()?;
    let repo = args.with_repo(config)?;
    tracing::info!("Repository cloned: {}", repo.path().display());
    let (cmd, _env) = args.to_docker_buildx_command_string()?;

    if args.dry_run {
        println!("cmd: {}", cmd);
        Ok(())
    } else {
        debug!("cmd: {}", cmd);
        let mut output = args.build_command()?;
        let output = output
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
}
