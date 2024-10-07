use anyhow::Context;
use git2::{
    build::RepoBuilder, BranchType, Cred, ErrorCode, FetchOptions, RemoteCallbacks, Repository,
};
use log::info;
use resolve_path::PathResolveExt;
use std::path::{Path, PathBuf};

use crate::{
    error::{SealedError, SealedResult},
    settings::Settings,
    util::{fs_utils::make_dirs, git_ops::parse_repo_name},
};

pub struct GitRepoService;

impl GitRepoService {
    pub fn fetch(repo: &str, branch_name: &str, settings: &Settings) -> SealedResult<Repository> {
        let repo = match GitRepoService::has_repo_been_cloned(repo, settings) {
            Ok(true) => GitRepoService::open_locally(repo, settings)?,
            Ok(false) => GitRepoService::clone_from_remote(repo, settings)?,
            Err(e) => return Err(e),
        };

        GitRepoService::checkout_branch(&repo, branch_name, settings)?;
        GitRepoService::update_from_remote(&repo, branch_name, settings)?;
        Ok(repo)
    }

    fn clone_from_remote(repo: &str, settings: &Settings) -> SealedResult<Repository> {
        let mut builder = Self::get_repo_builder(settings)?;

        let path = Self::resolve_repo_local(repo, settings)?;
        make_dirs(path.as_path())?;
        let repo = builder
            .clone(repo, path.as_path())
            .context("unable to clone git repository")?;

        Ok(repo)
    }

    fn get_fetch_options(settings: &Settings) -> SealedResult<FetchOptions> {
        let mut callbacks = RemoteCallbacks::new();

        if let Some(ssh_key) = settings.ssh_key.as_ref() {
            callbacks.credentials(|_url, username_from_url, _allowed_types| {
                let resolved_ssh_key = ssh_key.resolve();
                Cred::ssh_key(
                    username_from_url.unwrap(),
                    None,
                    Path::new(resolved_ssh_key.to_str().unwrap()),
                    None,
                )
            });
        }

        let mut fo = git2::FetchOptions::new();
        fo.remote_callbacks(callbacks);
        Ok(fo)
    }

    fn get_repo_builder(settings: &Settings) -> SealedResult<RepoBuilder> {
        // prepare bulder
        let fo = Self::get_fetch_options(settings)?;
        let mut builder = RepoBuilder::new();
        builder.fetch_options(fo);

        Ok(builder)
    }

    // Checkout a branch and set it as the current branch
    fn checkout_branch(
        repo: &Repository,
        branch_name: &str,
        _settings: &Settings,
    ) -> SealedResult<()> {
        let remote_name = "origin";
        let remote_branch_name = format!("{}/{}", remote_name, branch_name);

        info!("Checking out branch: {}", branch_name);

        // Try to find the branch locally or remotely
        let _branch = match repo.find_branch(branch_name, BranchType::Local) {
            Ok(branch) => branch,
            Err(_) => match repo.find_branch(&remote_branch_name, BranchType::Remote) {
                Ok(branch) => branch,
                Err(_) => {
                    // Branch doesn't exist locally or remotely, create it
                    tracing::info!("Branch '{}' not found. Creating it.", branch_name);
                    let head = repo.head()?;
                    let head_commit = head.peel_to_commit()?;
                    repo.branch(branch_name, &head_commit, false)?;
                    repo.find_branch(branch_name, BranchType::Local)?
                }
            },
        };

        let current_branch = GitRepoService::current_branch(repo)?;
        if !current_branch.contains(branch_name) {
            let obj = repo.revparse_single(&("refs/heads/".to_owned() + branch_name))?;
            repo.set_head(&("refs/heads/".to_owned() + branch_name))?;
            let mut checkout_builder = git2::build::CheckoutBuilder::new();
            checkout_builder.force();
            repo.checkout_tree(&obj, Some(&mut checkout_builder))?;
            info!("Checked out branch: {}", branch_name);
        }

        Ok(())
    }

    pub fn update_from_remote(
        repo: &Repository,
        branch_name: &str,
        settings: &Settings,
    ) -> SealedResult<()> {
        let remote_name = "origin";
        let mut fo = Self::get_fetch_options(settings)?;

        // Remove 'origin/' prefix if it exists in the branch_name
        let clean_branch_name = branch_name.strip_prefix("origin/").unwrap_or(branch_name);

        info!(
            "Fetching latest changes for branch '{}' from remote: {}",
            clean_branch_name, remote_name
        );

        // Fetch the latest changes from the remote for all branches
        repo.find_remote(remote_name)?.fetch(
            &["refs/heads/*:refs/remotes/origin/*"],
            Some(&mut fo),
            None,
        )?;

        info!("Fetching complete");

        // Get the remote branch reference
        let remote_ref_name = format!("refs/remotes/{}/{}", remote_name, clean_branch_name);
        let remote_ref = match repo.find_reference(&remote_ref_name) {
            Ok(r) => r,
            Err(_) => {
                tracing::warn!(
                    "Remote branch '{}' not found. It might be a new branch.",
                    clean_branch_name
                );
                // Create a new branch based on the current HEAD
                let head = repo.head()?;
                let head_commit = head.peel_to_commit()?;
                repo.branch(clean_branch_name, &head_commit, false)?;
                repo.find_reference(&format!("refs/heads/{}", clean_branch_name))?
            }
        };
        let remote_commit = repo.reference_to_annotated_commit(&remote_ref)?;

        info!("Using commit: {}", remote_commit.id());

        // Get or create the local branch reference
        let local_ref_name = format!("refs/heads/{}", clean_branch_name);
        let mut local_ref = match repo.find_reference(&local_ref_name) {
            Ok(r) => r,
            Err(_) => {
                // If local branch doesn't exist, create it
                repo.reference(
                    &local_ref_name,
                    remote_commit.id(),
                    true,
                    &format!("Create local branch {}", clean_branch_name),
                )?
            }
        };
        let local_commit = repo.reference_to_annotated_commit(&local_ref)?;

        info!("Local commit: {}", local_commit.id());

        // Perform merge analysis
        let analysis = repo.merge_analysis(&[&remote_commit])?;

        if analysis.0.is_up_to_date() {
            tracing::info!("Already up to date");
        } else if analysis.0.is_fast_forward() {
            // Fast-forward
            local_ref.set_target(
                remote_commit.id(),
                &format!(
                    "Fast-Forward {} to {}",
                    clean_branch_name,
                    remote_commit.id()
                ),
            )?;
            repo.set_head(&local_ref_name)?;
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
            tracing::info!(
                "Fast-forwarded {} to {}",
                clean_branch_name,
                remote_commit.id()
            );
        } else {
            // Normal merge
            let local_tree = repo.find_commit(local_commit.id())?.tree()?;
            let remote_tree = repo.find_commit(remote_commit.id())?.tree()?;
            let ancestor =
                repo.find_commit(repo.merge_base(local_commit.id(), remote_commit.id())?)?;
            let mut idx = repo.merge_trees(&ancestor.tree()?, &local_tree, &remote_tree, None)?;

            if idx.has_conflicts() {
                tracing::warn!("Merge conflicts detected. Aborting.");
                repo.checkout_index(Some(&mut idx), None)?;
                return Err(SealedError::Git2OperationFailed(git2::Error::from_str(
                    "Merge conflicts detected",
                )));
            }

            let result_tree = repo.find_tree(idx.write_tree_to(repo)?)?;
            let message = format!("Merge {} into {}", remote_commit.id(), clean_branch_name);
            let sig = repo.signature()?;
            let local_commit = repo.find_commit(local_commit.id())?;
            let remote_commit = repo.find_commit(remote_commit.id())?;
            let merge_commit = repo.commit(
                Some("HEAD"),
                &sig,
                &sig,
                &message,
                &result_tree,
                &[&local_commit, &remote_commit],
            )?;

            // Move the branch reference to the new commit
            local_ref.set_target(
                merge_commit,
                &format!("Merge {} into {}", remote_commit.id(), clean_branch_name),
            )?;
            repo.set_head(&local_ref_name)?;
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
            tracing::info!("Merged {} into {}", remote_commit.id(), clean_branch_name);
        }

        Ok(())
    }

    pub fn open_locally(repo: &str, settings: &Settings) -> SealedResult<Repository> {
        let path = Self::resolve_repo_local(repo, settings)?;
        tracing::info!("Resolved git repository to: {}", path.display());

        match Repository::open(path) {
            Ok(repo) => Ok(repo),
            Err(e) => Err(SealedError::Git2OperationFailed(e)),
        }
    }

    pub fn short_sha(repo: &Repository) -> SealedResult<String> {
        let head = repo.head()?;
        let commit = head.peel_to_commit()?;
        let short_sha = commit.id().to_string()[..7].to_string();

        Ok(short_sha)
    }

    fn current_branch(repo: &Repository) -> SealedResult<String> {
        let head = match repo.head() {
            Ok(head) => Some(head),
            Err(ref e)
                if e.code() == ErrorCode::UnbornBranch || e.code() == ErrorCode::NotFound =>
            {
                None
            }
            Err(e) => return Err(SealedError::Git2OperationFailed(e)),
        };
        let head = head.as_ref().and_then(|h| h.shorthand());

        Ok(head.unwrap_or("Not currently on any branch").to_string())
    }

    fn has_repo_been_cloned(repo: &str, settings: &Settings) -> SealedResult<bool> {
        let path = Self::resolve_repo_local(repo, settings)?;
        let dot_git_path = path.join(".git");
        Ok(dot_git_path.exists() && dot_git_path.is_dir())
    }

    fn resolve_repo_local(repo: &str, settings: &Settings) -> SealedResult<PathBuf> {
        let root_dir = &settings.working_directory;
        let repo_name = parse_repo_name(repo)?;
        let path = root_dir.join(repo_name);
        Ok(path)
    }
}
