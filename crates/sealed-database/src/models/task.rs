use crate::{
    error::SealedDatabaseError,
    taskfile::{command, location, user},
};
use sealed_common::{
    util::{
        cache::{combine, CryptoHash},
        format::CodeStr,
    },
    CACHE_VERSION,
};

use super::taskfile::TaskFile;

use {
    crate::error::SealedDatabaseResult,
    serde::{de::Error, Deserialize, Deserializer},
    std::{
        collections::HashMap,
        fmt::{self, Display, Formatter},
        path::PathBuf,
    },
    typed_path::UnixPathBuf,
};

// The default location for commands and files copied into the container
pub const DEFAULT_LOCATION: &str = "/scratch";

// The default user for commands and files copied into the container
pub const DEFAULT_USER: &str = "root";

// Deserializer for `UnixPathBuf`
pub fn deserialize_unix_path_buf<'de, D>(
    deserializer: D,
) -> SealedDatabaseResult<UnixPathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    UnixPathBuf::try_from(PathBuf::deserialize(deserializer)?)
        .map_err(|_| D::Error::custom("invalid path"))
}

// Deserializer for `Option<UnixPathBuf>`
fn deserialize_option_unix_path_buf<'de, D>(
    deserializer: D,
) -> Result<Option<UnixPathBuf>, D::Error>
where
    D: Deserializer<'de>,
{
    if let Some(path) = Option::<PathBuf>::deserialize(deserializer)? {
        match UnixPathBuf::try_from(path) {
            Ok(path) => Ok(Some(path)),
            Err(_) => Err(D::Error::custom("invalid path")),
        }
    } else {
        Ok(None)
    }
}

// Deserializer for `Vec<UnixPathBuf>`
fn deserialize_vec_unix_path_buf<'de, D>(deserializer: D) -> Result<Vec<UnixPathBuf>, D::Error>
where
    D: Deserializer<'de>,
{
    let mut result = Vec::new();
    for path in Vec::<PathBuf>::deserialize(deserializer)? {
        match UnixPathBuf::try_from(path) {
            Ok(path) => {
                result.push(path);
            }
            Err(_) => return Err(D::Error::custom("invalid path")),
        }
    }
    Ok(result)
}

// This struct represents a path on the host and a corresponding path in the container.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MappingPath {
    pub host_path: PathBuf,
    pub container_path: UnixPathBuf,
}

impl Display for MappingPath {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}:{}",
            self.host_path.to_string_lossy(),
            self.container_path.to_string_lossy(),
        )
    }
}

struct MappingPathVisitor;

impl<'de> serde::de::Visitor<'de> for MappingPathVisitor {
    type Value = MappingPath;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a path")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Some((host_path, container_path)) = v.split_once(':') {
            Ok(MappingPath {
                host_path: host_path
                    .parse()
                    .map_err(|_| E::custom("Illegal host path."))?,
                container_path: container_path
                    .parse()
                    .map_err(|_| E::custom("Illegal container path."))?,
            })
        } else {
            Ok(MappingPath {
                host_path: v.parse().map_err(|_| E::custom("Illegal path."))?,
                container_path: v.parse().map_err(|_| E::custom("Illegal path."))?,
            })
        }
    }
}

impl<'de> Deserialize<'de> for MappingPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(MappingPathVisitor)
    }
}

// This struct represents a task.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub description: Option<String>,

    // Must point to valid task names [ref:dependencies_exist] and the dependency DAG must not form
    // cycles [ref:tasks_dag]
    #[serde(default)]
    pub dependencies: Vec<String>,

    // Must be disabled if any of the following conditions hold:
    // - `mount_paths` is nonempty [ref:mount_paths_nand_cache]
    // - `ports` is nonempty [ref:ports_nand_cache]
    // - `extra_docker_arguments` is nonempty [ref:extra_docker_arguments_nand_cache]
    #[serde(default = "default_task_cache")]
    pub cache: bool,

    // Keys must not contain `=` [ref:env_var_equals]
    #[serde(default)] // [tag:default_environment]
    pub environment: HashMap<String, Option<String>>,

    // Must be relative [ref:input_paths_relative]
    #[serde(default, deserialize_with = "deserialize_vec_unix_path_buf")]
    pub input_paths: Vec<UnixPathBuf>,

    // Must be relative [ref:excluded_input_paths_relative]
    #[serde(default, deserialize_with = "deserialize_vec_unix_path_buf")]
    pub excluded_input_paths: Vec<UnixPathBuf>,

    // Must be relative [ref:output_paths_relative]
    #[serde(default, deserialize_with = "deserialize_vec_unix_path_buf")]
    pub output_paths: Vec<UnixPathBuf>,

    // Must be relative [ref:output_paths_on_failure_relative]
    #[serde(default, deserialize_with = "deserialize_vec_unix_path_buf")]
    pub output_paths_on_failure: Vec<UnixPathBuf>,

    // Can be relative or absolute (absolute paths are allowed in order to support mounting the
    //   Docker socket, which is usually located at `/var/run/docker.sock`)
    // Must not contain `,` [ref:mount_paths_no_commas]
    // Must be empty if `cache` is enabled [ref:mount_paths_nand_cache]
    // Can be `host_path:container_path` or a single path if `host_path` is the same as
    //   `container_path`
    #[serde(default)] // [tag:default_mount_paths]
    pub mount_paths: Vec<MappingPath>,

    #[serde(default = "default_task_mount_readonly")]
    pub mount_readonly: bool,

    // Must be empty if `cache` is enabled [ref:ports_nand_cache]
    #[serde(default)] // [tag:default_ports]
    pub ports: Vec<String>,

    // If `None`, the corresponding top-level value in the TaskFile should be used. There is a
    // helper function [ref:location_helper] which implements that logic. This path must be absolute
    // [ref:task_location_absolute].
    #[serde(default, deserialize_with = "deserialize_option_unix_path_buf")]
    pub location: Option<UnixPathBuf>,

    // If `None`, the corresponding top-level value in the TaskFile should be used. There is a
    // helper function [ref:user_helper] which implements that logic.
    pub user: Option<String>,

    // The actual command to run in the container is this value concatenated with the command prefix
    // (see below). There is a helper function [ref:command_helper] which implements that logic.
    #[serde(default)]
    pub command: String,

    // If `None`, the corresponding top-level value in the TaskFile should be used. There is a
    // helper function [ref:command_helper] which implements that logic.
    #[serde(default)]
    pub command_prefix: Option<String>,

    // Must be empty if `cache` is enabled [ref:extra_docker_arguments_nand_cache]
    #[serde(default)]
    pub extra_docker_arguments: Vec<String>,
}

fn default_task_cache() -> bool {
    true
}

pub fn default_task_mount_readonly() -> bool {
    false
}

// Check that a task is valid.
#[allow(clippy::too_many_lines)]
pub fn check_task(name: &str, task: &Task) -> SealedDatabaseResult<()> {
    // Check that environment variable names don't have `=` in them [tag:env_var_equals].
    for variable in task.environment.keys() {
        if variable.contains('=') {
            return Err(SealedDatabaseError::FailedToRunUserCommand(
                format!(
                    "Environment variable {} of task {} contains {}.",
                    variable.code_str(),
                    name.code_str(),
                    "=".code_str(),
                ),
                None,
            ));
        }
    }

    // Check that `input_paths` are relative [tag:input_paths_relative].
    for path in &task.input_paths {
        if !path.is_relative() {
            return Err(SealedDatabaseError::FailedToRunUserCommand(
                format!(
                    "Task {} has an absolute {}: {}.",
                    name.code_str(),
                    "input_path".code_str(),
                    path.to_string_lossy().code_str(),
                ),
                None,
            ));
        }
    }

    // Check that `excluded_input_paths` are relative [tag:excluded_input_paths_relative].
    for path in &task.excluded_input_paths {
        if !path.is_relative() {
            return Err(SealedDatabaseError::FailedToRunUserCommand(
                format!(
                    "Task {} has an absolute {}: {}.",
                    name.code_str(),
                    "excluded_input_path".code_str(),
                    path.to_string_lossy().code_str(),
                ),
                None,
            ));
        }
    }

    // Check that `output_paths` are relative [tag:output_paths_relative].
    for path in &task.output_paths {
        if !path.is_relative() {
            return Err(SealedDatabaseError::FailedToRunUserCommand(
                format!(
                    "Task {} has an absolute path in {}: {}.",
                    name.code_str(),
                    "output_paths".code_str(),
                    path.to_string_lossy().code_str(),
                ),
                None,
            ));
        }
    }

    // Check that `output_paths_on_failure` are relative [tag:output_paths_on_failure_relative].
    for path in &task.output_paths_on_failure {
        if !path.is_relative() {
            return Err(SealedDatabaseError::FailedToRunUserCommand(
                format!(
                    "Task {} has an absolute path in {}: {}.",
                    name.code_str(),
                    "output_paths_on_failure".code_str(),
                    path.to_string_lossy().code_str(),
                ),
                None,
            ));
        }
    }

    // Check `mount_paths`.
    for path in &task.mount_paths {
        // Check that the path doesn't contain any commas [tag:mount_paths_no_commas].
        if path.container_path.to_string_lossy().contains(',')
            || path.host_path.to_string_lossy().contains(',')
        {
            return Err(SealedDatabaseError::FailedToRunUserCommand(
                format!(
                    "Mount path {} of task {} has a {}.",
                    format!("{path}").code_str(),
                    name.code_str(),
                    ",".code_str(),
                ),
                None,
            ));
        }
    }

    // Check that `location` is absolute [tag:task_location_absolute].
    if let Some(location) = &task.location {
        if !location.is_absolute() {
            return Err(SealedDatabaseError::FailedToRunUserCommand(
                format!(
                    "Task {} has a relative {}: {}.",
                    name.code_str(),
                    "location".code_str(),
                    location.to_string_lossy().code_str(),
                ),
                None,
            ));
        }
    }

    // If a task has any mount paths, then caching should be disabled [tag:mount_paths_nand_cache].
    if !task.mount_paths.is_empty() && task.cache {
        return Err(SealedDatabaseError::FailedToRunUserCommand(
            format!(
                "Task {} has {} but does not disable caching. \
             To fix this, set {} for this task.",
                name.code_str(),
                "mount_paths".code_str(),
                "cache: false".code_str(),
            ),
            None,
        ));
    }

    // If a task exposes ports, then caching should be disabled [tag:ports_nand_cache].
    if !&task.ports.is_empty() && task.cache {
        return Err(SealedDatabaseError::FailedToRunUserCommand(
            format!(
                "Task {} exposes ports but does not disable caching. \
             To fix this, set {} for this task.",
                name.code_str(),
                "cache: false".code_str(),
            ),
            None,
        ));
    }

    // If a task has any extra Docker arguments, then caching should be disabled.
    // [tag:extra_docker_arguments_nand_cache]
    if !&task.extra_docker_arguments.is_empty() && task.cache {
        return Err(SealedDatabaseError::FailedToRunUserCommand(
            format!(
                "Task {} has extra Docker arguments but does not disable caching. \
             To fix this, set {} for this task.",
                name.code_str(),
                "cache: false".code_str(),
            ),
            None,
        ));
    }

    // If we made it this far, the task is valid.
    Ok(())
}

// Determine the image name for a task based on the name of the image for the previous task in the
// schedule (or the base image, if this is the first task).
pub fn image_name(
    previous_image: &str,
    docker_repo: &str,
    taskfile: &TaskFile,
    task: &Task,
    input_files_hash: &str,
    environment: &HashMap<String, String>,
) -> String {
    // Compute the command for this task.
    let command = command(taskfile, task);

    // If there are no environment variables, no input paths, and no command to run, we can just use
    // the image from the previous task.
    if task.environment.is_empty() && task.input_paths.is_empty() && command.is_empty() {
        return previous_image.to_owned();
    }

    // Start with a hash of the cache version.
    let mut cache_key: String = format!("{}", CACHE_VERSION).crypto_hash();

    // Incorporate the previous image.
    cache_key = combine(&cache_key, previous_image);

    // Incorporate the environment variables.
    let mut environment_hash = String::new();
    let mut variables = task.environment.keys().collect::<Vec<_>>();
    variables.sort();
    for variable in variables {
        // The variable name
        environment_hash = combine(&environment_hash, variable);

        // The value [ref:environment_valid]
        environment_hash = combine(&environment_hash, &environment[variable]);
    }
    cache_key = combine(&cache_key, &environment_hash);

    // Incorporate the input paths and contents.
    cache_key = combine(&cache_key, input_files_hash);

    // Incorporate the location.
    cache_key = combine(&cache_key, &location(taskfile, task));

    // Incorporate the user.
    cache_key = combine(&cache_key, &user(taskfile, task));

    // Incorporate the command.
    cache_key = combine(&cache_key, &command);

    // We add this "task-" prefix because Docker has a rule that tags cannot be 64-byte hexadecimal
    // strings. See this for more details: https://github.com/moby/moby/issues/20972
    format!("{docker_repo}:task-{cache_key}")
}
