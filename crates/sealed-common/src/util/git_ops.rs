use crate::error::SealedResult;
use std::path::PathBuf;
use tokio::process::Command;

pub fn parse_repo_name(url: &str) -> SealedResult<String> {
    let parsed = parse_git_url(url)?;
    Ok(parsed.name)
}

fn parse_git_url(url: &str) -> SealedResult<git_url_parse::GitUrl> {
    let parsed = git_url_parse::GitUrl::parse(url)?;
    Ok(parsed)
}

// pub async fn clone_repository(repo_url: &str, target_dir: &PathBuf) -> SealedResult<()> {
//     let output = Command::new("git")
//         .arg("clone")
//         .arg(repo_url)
//         .arg(target_dir)
//         .output()
//         .await?;

//     if !output.status.success() {
//         return Err(crate::error::SealedError::GitOperationFailed(
//             String::from_utf8_lossy(&output.stderr).to_string(),
//         ));
//     }

//     Ok(())
// }

// pub async fn process_git_push(repo_url: &str) -> SealedResult<PathBuf> {
//     let temp_dir = tempfile::tempdir()?;
//     let target_dir = temp_dir.path().to_path_buf();

//     clone_repository(repo_url, &target_dir).await?;

//     // Detect project type and configuration
//     let project_type = detect_project_type(&target_dir)?;
//     println!("Project type: {:?}", project_type);

//     // Process Kubernetes resources
//     // process_k8s_resources(&target_dir, &project_type).await?;

//     Ok(target_dir)
// }

// fn detect_project_type(target_dir: &PathBuf) -> SealedResult<ProjectType> {
//     if target_dir.join("package.json").exists() {
//         Ok(ProjectType::JavaScript)
//     } else if target_dir.join("requirements.txt").exists() {
//         Ok(ProjectType::Python)
//     } else if target_dir.join("pom.xml").exists() {
//         Ok(ProjectType::Java)
//     } else if target_dir.join("Cargo.toml").exists() {
//         Ok(ProjectType::Rust)
//     } else {
//         Err(crate::error::SealedError::UnsupportedProjectType)
//     }
// }

// #[derive(Debug)]
// pub enum ProjectType {
//     JavaScript,
//     Python,
//     Java,
//     Rust,
// }
