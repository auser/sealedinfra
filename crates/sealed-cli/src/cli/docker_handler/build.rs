#![allow(unused)]
use anyhow::Context;
use console::{style, Emoji};
use git2::{
    build::{CheckoutBuilder, RepoBuilder},
    BranchType, Cred, ErrorCode, FetchOptions, RemoteCallbacks, Repository,
};
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, error, info, warn};
use rand::Rng;
use resolve_path::PathResolveExt;
use sealed_common::settings::Settings;
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::process::Command;

use crate::error::{SealedCliError, SealedCliResult};

use super::DockerHandlerArgs;

use futures::future::join_all;
use tokio::io::{AsyncBufReadExt, BufReader};

static LEVER: Emoji<'_, '_> = Emoji("🍴 ", "");
static SCREWDRIVER: Emoji<'_, '_> = Emoji("🪛 ", "");
static TRUCK: Emoji<'_, '_> = Emoji("🚚  ", "");

pub async fn run(args: &mut DockerHandlerArgs, config: &Settings) -> SealedCliResult<()> {
    let mut rng = rand::thread_rng();

    let spinner_style = ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
        .unwrap()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");

    let count = rng.gen_range(30..80);
    let pb = ProgressBar::new(count);
    pb.set_style(spinner_style);

    println!(
        "{} Fetching repository: {}",
        style("[1/3]").bold().dim(),
        LEVER
    );

    let repo = args.with_repo(config)?;
    info!("Repository cloned: {}", repo.path().display());

    println!(
        "{} Building docker command: {}",
        style("[2/3]").bold().dim(),
        TRUCK
    );

    let cmd = args.to_docker_buildx_command_string(config)?;
    let env_prefix = args.get_env_prefix();

    if args.dry_run {
        println!("cmd: {}", cmd);
        Ok(())
    } else {
        debug!("cmd: {}", cmd);
        let mut command = args.build_command(config)?;

        for env_var in env_prefix.iter() {
            let parts: Vec<&str> = env_var.splitn(2, '=').collect();
            if parts.len() == 2 {
                command.env(parts[0], parts[1]);
            }
        }
        command.env("DOCKER_BUILDKIT", "1");

        // Ensure we can capture stdout and stderr
        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::piped());

        println!(
            "{} Building docker image: {}",
            style("[3/3]").bold().dim(),
            SCREWDRIVER
        );

        let mut child = command
            .spawn()
            .map_err(|e| SealedCliError::Runtime(e.to_string()))?;

        let stdout = child.stdout.take().expect("Failed to capture stdout");
        let stderr = child.stderr.take().expect("Failed to capture stderr");

        let stdout_handle = tokio::spawn({
            let pb = pb.clone();
            async move {
                let mut reader = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    // println!(
                    //     "{} stdout: {}",
                    //     style(format!("stdout: {}", line)).bold().dim(),
                    //     TRUCK
                    // );
                    pb.set_message(format!("stdout: {}", line));
                    pb.inc(1);
                }
            }
        });

        let stderr_handle = tokio::spawn({
            let pb = pb.clone();
            async move {
                let mut reader = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    // println!(
                    //     "{} err: {}",
                    //     style(format!("stderr: {}", line)).red(),
                    //     TRUCK
                    // );
                    pb.set_message(format!("stderr: {}", line));
                    pb.inc(1);
                }
            }
        });

        // Tick the progress bar
        let progress_handle = tokio::spawn({
            let pb = pb.clone();
            async move {
                while !pb.is_finished() {
                    pb.tick();
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        });

        // Wait for the command to complete and the output streams to be processed
        let (result, _, _) = tokio::join!(child.wait(), stdout_handle, stderr_handle);

        pb.finish_and_clear();

        match result {
            Ok(status) if status.success() => Ok(()),
            _ => Err(SealedCliError::Runtime(
                "Docker build command failed".to_string(),
            )),
        }
    }
}
