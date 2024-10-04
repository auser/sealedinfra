#![allow(unused)]
use anyhow::Context;
use git2::{
    build::{CheckoutBuilder, RepoBuilder},
    BranchType, Cred, ErrorCode, FetchOptions, RemoteCallbacks, Repository,
};
use resolve_path::PathResolveExt;
use std::path::{Path, PathBuf};
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
    println!("cmd: {}", cmd);

    Ok(())
}
