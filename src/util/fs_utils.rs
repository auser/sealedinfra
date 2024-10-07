use std::path::{Path, PathBuf};

use crate::error::SealedResult;

pub fn make_dirs(path: &Path) -> SealedResult<()> {
    tracing::debug!("Creating directories: {}", path.display());
    std::fs::create_dir_all(path)?;
    tracing::debug!("Created directories: {}", path.display());
    Ok(())
}

pub fn find_file_by_name(path: &Path, filename: &str) -> SealedResult<PathBuf> {
    find_file_by_name_recursive(path, filename)
}

// Find all files with the given name in the given directory and its subdirectories
// and return a vector of paths.
pub fn find_multiple_files_by_name(path: &Path, filenames: &[&str]) -> SealedResult<Vec<PathBuf>> {
    find_multiple_files_by_name_recursive(path, filenames)
}

fn find_file_by_name_recursive(root: &Path, filename: &str) -> SealedResult<PathBuf> {
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if let Ok(found_path) = find_file_by_name_recursive(&path, filename) {
                return Ok(found_path);
            }
        } else if path.file_name().and_then(|s| s.to_str()) == Some(filename) {
            return Ok(path);
        }
    }
    Err(crate::error::SealedError::FileNotFound(
        filename.to_string(),
    ))
}

// find all files with the given name in the given directory and its subdirectories
// and return a vector of paths.
fn find_multiple_files_by_name_recursive(
    root: &Path,
    filenames: &[&str],
) -> SealedResult<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            paths.extend(find_multiple_files_by_name_recursive(&path, filenames)?);
        } else if filenames.contains(&path.file_name().and_then(|s| s.to_str()).unwrap()) {
            paths.push(path);
        }
    }
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use super::*;

    #[test]
    fn test_find_file_by_name() {
        let temp_root = generate_test_data(false).unwrap();
        let expected_path = temp_root.join("tests/test_data/subdir").join("test.txt");
        let found_path = find_file_by_name(&temp_root, "test.txt").unwrap();
        assert_eq!(found_path, expected_path);
    }

    #[test]
    fn test_find_multiple_files_by_name() {
        let temp_root = generate_test_data(false).unwrap();
        let second_expected_path = temp_root.join("Dockerfile");
        let expected_path = temp_root.join("tests/test_data/subdir").join("test.txt");
        let found_paths =
            find_multiple_files_by_name(&temp_root, &["test.txt", "Dockerfile"]).unwrap();
        // Assertion works because Dockerfile is found before a deeper test.txt
        assert_eq!(found_paths, vec![second_expected_path, expected_path]);
    }

    fn generate_test_data(create_git_repo: bool) -> SealedResult<PathBuf> {
        // Create a directory structure like:
        // tests/test_data/subdir/test.txt
        let temp_root = tempfile::tempdir()?;
        let temp_root_path = temp_root.into_path();
        let path = temp_root_path.join("tests/test_data/subdir");
        std::fs::create_dir_all(&path)?;
        let file_path = path.join("test.txt");
        std::fs::write(&file_path, "test content")?;
        // Write Dockerfile to the root directory
        let dockerfile_path = temp_root_path.join("Dockerfile");
        std::fs::write(
            &dockerfile_path,
            "FROM alpine:latest\nRUN echo 'test content' > /test.txt\n",
        )?;

        // Add a bunch of random files and directories to the directory
        for i in 0..10 {
            // Add a random directory
            let random_dir_path = path.join(format!("random_dir_{}", i));
            std::fs::create_dir_all(&random_dir_path)?;

            // Add a random file
            let random_file_path = random_dir_path.join(format!("random_file_{}.txt", i));
            std::fs::write(&random_file_path, format!("random content {}", i))?;
        }

        // Initialize a git repository
        if create_git_repo {
            initialize_test_git_repo(&path)?;
        }

        Ok(temp_root_path)
    }

    fn initialize_test_git_repo(path: &Path) -> SealedResult<()> {
        let repo_path = path.join("repo");
        std::fs::create_dir(&repo_path)?;
        let output = Command::new("git")
            .arg("init")
            .current_dir(&repo_path)
            .output()?;
        assert!(output.status.success());
        // Add and commit all files
        let output = Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(&repo_path)
            .output()?;
        assert!(output.status.success());
        let output = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg("Initial commit")
            .current_dir(&repo_path)
            .output()?;
        assert!(output.status.success());
        Ok(())
    }
}
