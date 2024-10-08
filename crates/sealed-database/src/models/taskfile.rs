use std::collections::{HashMap, HashSet};

use sealed_common::{format::series, util::format::CodeStr};
use serde::Deserialize;
use typed_path::{UnixPath, UnixPathBuf};

use crate::error::{SealedDatabaseError, SealedDatabaseResult};

use super::task::{check_task, Task, DEFAULT_LOCATION, DEFAULT_USER};

// This struct represents a TaskFile.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TaskFile {
    pub image: String,

    // If present, must point to a task [ref:valid_default]
    pub default: Option<String>,

    // Must be absolute [ref:TaskFile_location_absolute]
    #[serde(default = "default_location")]
    #[serde(deserialize_with = "crate::models::task::deserialize_unix_path_buf")]
    pub location: UnixPathBuf,

    #[serde(default = "default_user")]
    pub user: String,

    #[serde(default)]
    pub command_prefix: String,

    #[serde(default)]
    pub tasks: HashMap<String, Task>,
}

fn default_location() -> UnixPathBuf {
    UnixPath::new(DEFAULT_LOCATION).to_owned()
}

fn default_user() -> String {
    DEFAULT_USER.to_owned()
}

// Parse config data.
pub fn parse(task_file_data: &str) -> SealedDatabaseResult<TaskFile> {
    // Deserialize the data.
    let task_file: TaskFile = serde_yaml::from_str(task_file_data)
        .map_err(|e| SealedDatabaseError::System(format!("{e}"), None))?;

    // Make sure the dependencies are valid.
    check_dependencies(&task_file)?;

    // Check that `location` is absolute [tag:TaskFile_location_absolute].
    if !task_file.location.is_absolute() {
        return Err(SealedDatabaseError::FailedToRunUserCommand(
            format!(
                "TaskFile has a relative {}: {}.",
                "location".code_str(),
                task_file.location.to_string_lossy().code_str(),
            ),
            None,
        ));
    }

    // Make sure each task is valid.
    for (name, task) in &task_file.tasks {
        check_task(name, task)?;
    }

    // Return the TaskFile.
    Ok(task_file)
}

// Fetch the variables for a task from the environment.
pub fn environment(task: &Task) -> Result<HashMap<String, String>, Vec<&str>> {
    // The result will be a map from variable name to value.
    let mut result = HashMap::new();

    // We accumulate a list of errors to be shown to the user when there is a problem.
    let mut violations = vec![];

    // Fetch each environment variable.
    for (arg, default) in &task.environment {
        // Read the variable from the environment.
        let maybe_var = std::env::var(arg);

        // If a default value was provided, use that if the variable is missing from the
        // environment. If there was no default, the variable must be in the environment or else
        // we'll report a violation.
        if let Some(default) = default {
            result.insert(arg.clone(), maybe_var.unwrap_or_else(|_| default.clone()));
        } else if let Ok(var) = maybe_var {
            result.insert(arg.clone(), var);
        } else {
            violations.push(arg.as_ref());
        }
    }

    // If there were no violations, return the map. Otherwise, report the violations.
    if violations.is_empty() {
        Ok(result)
    } else {
        Err(violations)
    }
}

// [tag:location_helper] Fetch the location for a task, defaulting to the top-level location if
// needed.
pub fn location(task_file: &TaskFile, task: &Task) -> UnixPathBuf {
    task.location
        .clone()
        .unwrap_or_else(|| task_file.location.clone())
}

// [tag:user_helper] Fetch the user for a task, defaulting to the top-level location if needed.
pub fn user(task_file: &TaskFile, task: &Task) -> String {
    task.user.clone().unwrap_or_else(|| task_file.user.clone())
}

// [tag:command_helper] Fetch the command for a task, including the prefix, using the top-level
// prefix if needed.
pub fn command(task_file: &TaskFile, task: &Task) -> String {
    let mut command = String::new();

    if let Some(command_prefix) = &task.command_prefix {
        command.push_str(command_prefix);
    } else {
        command.push_str(&task_file.command_prefix);
    }

    if !command.is_empty() && !task.command.is_empty() {
        command.push('\n');
    }

    command.push_str(&task.command);

    command
}

// Check that all dependencies exist and form a DAG (no cycles).
#[allow(clippy::too_many_lines)]
fn check_dependencies<'a>(task_file: &'a TaskFile) -> SealedDatabaseResult<()> {
    // Check the default task [tag:valid_default].
    let valid_default = task_file
        .default
        .as_ref()
        .map_or(true, |default| task_file.tasks.contains_key(default));

    // Map from task to vector of invalid dependencies.
    let mut violations: HashMap<String, Vec<String>> = HashMap::new();

    // Scan for invalid dependencies [tag:task_valid].
    for task in task_file.tasks.keys() {
        // [ref:task_valid]
        for dependency in &task_file.tasks[task].dependencies {
            if !task_file.tasks.contains_key(dependency) {
                // [tag:dependencies_exist]
                violations
                    .entry(task.clone())
                    .or_default()
                    .push(dependency.clone());
            }
        }
    }

    // If there were any invalid dependencies, report them.
    if !violations.is_empty() {
        let violations_series = series(
            violations
                .iter()
                .map(|(task, dependencies)| {
                    format!(
                        "{} ({})",
                        task.code_str(),
                        series(
                            dependencies
                                .iter()
                                .map(|task| format!("{}", task.code_str()))
                                .collect::<Vec<_>>()
                                .as_ref(),
                        ),
                    )
                })
                .collect::<Vec<_>>()
                .as_ref(),
        );

        if valid_default {
            return Err(SealedDatabaseError::FailedToRunUserCommand(
                format!("The following tasks have invalid dependencies: {violations_series}."),
                None,
            ));
        }

        return Err(SealedDatabaseError::FailedToRunUserCommand(
            format!(
                "The default task {} does not exist, and the following tasks have invalid \
             dependencies: {}.",
                task_file.default.as_ref().unwrap().code_str(), // [ref:valid_default]
                violations_series,
            ),
            None,
        ));
    } else if !valid_default {
        return Err(SealedDatabaseError::FailedToRunUserCommand(
            format!(
                "The default task {} does not exist.",
                task_file.default.as_ref().unwrap().code_str(), // [ref:valid_default]
            ),
            None,
        ));
    }

    // Check that the dependencies aren't cyclic [tag:tasks_dag].
    let mut visited: HashSet<&'a str> = HashSet::new();
    for task in task_file.tasks.keys() {
        let mut frontier: Vec<(&'a str, usize)> = vec![(task, 0)];
        let mut ancestors_set: HashSet<&'a str> = HashSet::new();
        let mut ancestors_stack: Vec<&'a str> = vec![];

        // Keep going as long as there are more nodes to process [tag:TaskFile_frontier_nonempty].
        while let Some((task, task_depth)) = frontier.pop() {
            // Take the top task from the frontier. This is safe due to
            // [ref:TaskFile_frontier_nonempty].

            // Update the ancestors set and stack.
            for _ in 0..ancestors_stack.len() - task_depth {
                // The `unwrap` is safe because `ancestors_stack.len()` is positive in every
                // iteration of this loop.
                let task_to_remove = ancestors_stack.pop().unwrap();
                ancestors_set.remove(task_to_remove);
            }

            // If this task is an ancestor of itself, we have a cycle. Return an error.
            if ancestors_set.contains(task) {
                let mut cycle_iter = ancestors_stack.iter();
                cycle_iter.find(|&&x| x == task);
                let mut cycle = cycle_iter.collect::<Vec<_>>();
                cycle.push(&task); // [tag:cycle_nonempty]
                let error_message = if cycle.len() == 1 {
                    format!("{} depends on itself.", cycle[0].code_str())
                } else if cycle.len() == 2 {
                    format!(
                        "{} and {} depend on each other.",
                        cycle[0].code_str(),
                        cycle[1].code_str(),
                    )
                } else {
                    let mut cycle_dependencies = cycle[1..].to_owned();
                    cycle_dependencies.push(cycle[0]); // [ref:cycle_nonempty]
                    format!(
                        "{}.",
                        series(
                            cycle
                                .iter()
                                .zip(cycle_dependencies)
                                .map(|(x, y)| {
                                    format!("{} depends on {}", x.code_str(), y.code_str())
                                })
                                .collect::<Vec<_>>()
                                .as_ref(),
                        ),
                    )
                };
                return Err(SealedDatabaseError::FailedToRunUserCommand(
                    format!("The dependencies are cyclic. {error_message}"),
                    None,
                ));
            }

            // If we've never seen this task before, add its dependencies to the frontier.
            if !visited.contains(task) {
                visited.insert(task);

                ancestors_set.insert(task);
                ancestors_stack.push(task);

                for dependency in &task_file.tasks[task].dependencies {
                    frontier.push((dependency, task_depth + 1));
                }
            }
        }
    }

    // No violations
    Ok(())
}

#[cfg(test)]
mod tests {
    use {
        super::{
            check_dependencies, check_task, command, environment, location, parse, user, Task,
            TaskFile, DEFAULT_LOCATION, DEFAULT_USER,
        },
        crate::task::{image_name, MappingPath},
        std::{collections::HashMap, env, path::Path},
        typed_path::UnixPath,
    };

    #[test]
    fn parse_empty() {
        let input = r"
image: encom:os-12
"
        .trim();

        let task_file = TaskFile {
            image: "encom:os-12".to_owned(),
            default: None,
            location: UnixPath::new(DEFAULT_LOCATION).to_owned(),
            user: DEFAULT_USER.to_owned(),
            command_prefix: String::new(),
            tasks: HashMap::new(),
        };

        assert_eq!(parse(input).unwrap(), task_file);
    }

    #[test]
    fn parse_minimal_task() {
        let input = r"
image: encom:os-12
tasks:
foo: {}
"
        .trim();

        let mut tasks = HashMap::new();
        tasks.insert(
            "foo".to_owned(),
            Task {
                description: None,
                dependencies: vec![],
                cache: true,
                environment: HashMap::new(),
                input_paths: vec![],
                excluded_input_paths: vec![],
                output_paths: vec![],
                output_paths_on_failure: vec![],
                mount_paths: vec![],
                mount_readonly: false,
                ports: vec![],
                location: None,
                user: None,
                command: String::new(),
                command_prefix: None,
                extra_docker_arguments: vec![],
            },
        );

        let task_file = TaskFile {
            image: "encom:os-12".to_owned(),
            default: None,
            location: UnixPath::new(DEFAULT_LOCATION).to_owned(),
            user: DEFAULT_USER.to_owned(),
            command_prefix: String::new(),
            tasks,
        };

        assert_eq!(parse(input).unwrap(), task_file);
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn parse_comprehensive_task() {
        let input = r"
image: encom:os-12
default: bar
location: /default_location
user: default_user
command_prefix: prefix
tasks:
foo: {}
bar:
description: Reticulate splines.
dependencies:
  - foo
cache: false
environment:
  SPAM: monty
  HAM: null
  EGGS: null
input_paths:
  - qux
  - quux
  - quuz
excluded_input_paths:
  - spam
  - ham
  - eggs
output_paths:
  - corge
  - grault
  - garply
output_paths_on_failure:
  - fnord
  - smurf
  - xyzzy
mount_paths:
  - wibble
  - /wobble
  - wubble:wabble
mount_readonly: true
ports:
  - 3000
  - 3001
  - 3002
location: /code
user: waldo
command: flob
command_prefix: flob_prefix
extra_docker_arguments:
  - --cpus
  - '4'
"
        .trim();

        let mut environment = HashMap::new();
        environment.insert("SPAM".to_owned(), Some("monty".to_owned()));
        environment.insert("HAM".to_owned(), None);
        environment.insert("EGGS".to_owned(), None);

        let mut tasks = HashMap::new();
        tasks.insert(
            "foo".to_owned(),
            Task {
                description: None,
                dependencies: vec![],
                cache: true,
                environment: HashMap::new(),
                input_paths: vec![],
                excluded_input_paths: vec![],
                output_paths: vec![],
                output_paths_on_failure: vec![],
                mount_paths: vec![],
                mount_readonly: false,
                ports: vec![],
                location: None,
                user: None,
                command: String::new(),
                command_prefix: None,
                extra_docker_arguments: vec![],
            },
        );
        tasks.insert(
            "bar".to_owned(),
            Task {
                description: Some("Reticulate splines.".to_owned()),
                dependencies: vec!["foo".to_owned()],
                cache: false,
                environment,
                input_paths: vec![
                    UnixPath::new("qux").to_owned(),
                    UnixPath::new("quux").to_owned(),
                    UnixPath::new("quuz").to_owned(),
                ],
                excluded_input_paths: vec![
                    UnixPath::new("spam").to_owned(),
                    UnixPath::new("ham").to_owned(),
                    UnixPath::new("eggs").to_owned(),
                ],
                output_paths: vec![
                    UnixPath::new("corge").to_owned(),
                    UnixPath::new("grault").to_owned(),
                    UnixPath::new("garply").to_owned(),
                ],
                output_paths_on_failure: vec![
                    UnixPath::new("fnord").to_owned(),
                    UnixPath::new("smurf").to_owned(),
                    UnixPath::new("xyzzy").to_owned(),
                ],
                mount_paths: vec![
                    MappingPath {
                        host_path: Path::new("wibble").to_owned(),
                        container_path: UnixPath::new("wibble").to_owned(),
                    },
                    MappingPath {
                        host_path: Path::new("/wobble").to_owned(),
                        container_path: UnixPath::new("/wobble").to_owned(),
                    },
                    MappingPath {
                        host_path: Path::new("wubble").to_owned(),
                        container_path: UnixPath::new("wabble").to_owned(),
                    },
                ],
                mount_readonly: true,
                ports: vec!["3000".to_owned(), "3001".to_owned(), "3002".to_owned()],
                location: Some(UnixPath::new("/code").to_owned()),
                user: Some("waldo".to_owned()),
                command: "flob".to_owned(),
                command_prefix: Some("flob_prefix".to_owned()),
                extra_docker_arguments: vec!["--cpus".to_owned(), "4".to_owned()],
            },
        );

        let task_file = TaskFile {
            image: "encom:os-12".to_owned(),
            default: Some("bar".to_owned()),
            location: UnixPath::new("/default_location").to_owned(),
            user: "default_user".to_owned(),
            command_prefix: "prefix".to_owned(),
            tasks,
        };

        assert_eq!(parse(input).unwrap(), task_file);
    }

    #[test]
    fn check_dependencies_valid_default() {
        let mut tasks = HashMap::new();
        tasks.insert(
            "foo".to_owned(),
            Task {
                description: None,
                dependencies: vec![],
                cache: true,
                environment: HashMap::new(),
                input_paths: vec![],
                excluded_input_paths: vec![],
                output_paths: vec![],
                output_paths_on_failure: vec![],
                mount_paths: vec![],
                mount_readonly: false,
                ports: vec![],
                location: None,
                user: None,
                command: String::new(),
                command_prefix: None,
                extra_docker_arguments: vec![],
            },
        );

        let task_file = TaskFile {
            image: "encom:os-12".to_owned(),
            default: Some("foo".to_owned()),
            location: UnixPath::new(DEFAULT_LOCATION).to_owned(),
            user: DEFAULT_USER.to_owned(),
            command_prefix: String::new(),
            tasks,
        };

        assert!(check_dependencies(&task_file).is_ok());
    }

    #[test]
    fn check_dependencies_invalid_default() {
        let mut tasks = HashMap::new();
        tasks.insert(
            "foo".to_owned(),
            Task {
                description: None,
                dependencies: vec![],
                cache: true,
                environment: HashMap::new(),
                input_paths: vec![],
                excluded_input_paths: vec![],
                output_paths: vec![],
                output_paths_on_failure: vec![],
                mount_paths: vec![],
                mount_readonly: false,
                ports: vec![],
                location: None,
                user: None,
                command: String::new(),
                command_prefix: None,
                extra_docker_arguments: vec![],
            },
        );

        let task_file = TaskFile {
            image: "encom:os-12".to_owned(),
            default: Some("bar".to_owned()),
            location: UnixPath::new(DEFAULT_LOCATION).to_owned(),
            user: DEFAULT_USER.to_owned(),
            command_prefix: String::new(),
            tasks,
        };

        let result = check_dependencies(&task_file);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("bar"));
    }

    #[test]
    fn check_dependencies_empty() {
        let task_file = TaskFile {
            image: "encom:os-12".to_owned(),
            default: None,
            location: UnixPath::new(DEFAULT_LOCATION).to_owned(),
            user: DEFAULT_USER.to_owned(),
            command_prefix: String::new(),
            tasks: HashMap::new(),
        };

        assert!(check_dependencies(&task_file).is_ok());
    }

    #[test]
    fn check_dependencies_single() {
        let mut tasks = HashMap::new();
        tasks.insert(
            "foo".to_owned(),
            Task {
                description: None,
                dependencies: vec![],
                cache: true,
                environment: HashMap::new(),
                input_paths: vec![],
                excluded_input_paths: vec![],
                output_paths: vec![],
                output_paths_on_failure: vec![],
                mount_paths: vec![],
                mount_readonly: false,
                ports: vec![],
                location: None,
                user: None,
                command: String::new(),
                command_prefix: None,
                extra_docker_arguments: vec![],
            },
        );

        let task_file = TaskFile {
            image: "encom:os-12".to_owned(),
            default: None,
            location: UnixPath::new(DEFAULT_LOCATION).to_owned(),
            user: DEFAULT_USER.to_owned(),
            command_prefix: String::new(),
            tasks,
        };

        assert!(check_dependencies(&task_file).is_ok());
    }

    #[test]
    fn check_task_dependencies_nonempty() {
        let mut tasks = HashMap::new();
        tasks.insert(
            "foo".to_owned(),
            Task {
                description: None,
                dependencies: vec![],
                cache: true,
                environment: HashMap::new(),
                input_paths: vec![],
                excluded_input_paths: vec![],
                output_paths: vec![],
                output_paths_on_failure: vec![],
                mount_paths: vec![],
                mount_readonly: false,
                ports: vec![],
                location: None,
                user: None,
                command: String::new(),
                command_prefix: None,
                extra_docker_arguments: vec![],
            },
        );
        tasks.insert(
            "bar".to_owned(),
            Task {
                description: None,
                dependencies: vec!["foo".to_owned()],
                cache: true,
                environment: HashMap::new(),
                input_paths: vec![],
                excluded_input_paths: vec![],
                output_paths: vec![],
                output_paths_on_failure: vec![],
                mount_paths: vec![],
                mount_readonly: false,
                ports: vec![],
                location: None,
                user: None,
                command: String::new(),
                command_prefix: None,
                extra_docker_arguments: vec![],
            },
        );

        let task_file = TaskFile {
            image: "encom:os-12".to_owned(),
            default: None,
            location: UnixPath::new(DEFAULT_LOCATION).to_owned(),
            user: DEFAULT_USER.to_owned(),
            command_prefix: String::new(),
            tasks,
        };

        assert!(check_dependencies(&task_file).is_ok());
    }

    #[test]
    fn check_dependencies_nonexistent() {
        let mut tasks = HashMap::new();
        tasks.insert(
            "foo".to_owned(),
            Task {
                description: None,
                dependencies: vec![],
                cache: true,
                environment: HashMap::new(),
                input_paths: vec![],
                excluded_input_paths: vec![],
                output_paths: vec![],
                output_paths_on_failure: vec![],
                mount_paths: vec![],
                mount_readonly: false,
                ports: vec![],
                location: None,
                user: None,
                command: String::new(),
                command_prefix: None,
                extra_docker_arguments: vec![],
            },
        );
        tasks.insert(
            "bar".to_owned(),
            Task {
                description: None,
                dependencies: vec!["foo".to_owned(), "baz".to_owned()],
                cache: true,
                environment: HashMap::new(),
                input_paths: vec![],
                excluded_input_paths: vec![],
                output_paths: vec![],
                output_paths_on_failure: vec![],
                mount_paths: vec![],
                mount_readonly: false,
                ports: vec![],
                location: None,
                user: None,
                command: String::new(),
                command_prefix: None,
                extra_docker_arguments: vec![],
            },
        );

        let task_file = TaskFile {
            image: "encom:os-12".to_owned(),
            default: None,
            location: UnixPath::new(DEFAULT_LOCATION).to_owned(),
            user: DEFAULT_USER.to_owned(),
            command_prefix: String::new(),
            tasks,
        };

        let result = check_dependencies(&task_file);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("baz"));
    }

    #[test]
    fn check_dependencies_cycle_1() {
        let mut tasks = HashMap::new();
        tasks.insert(
            "foo".to_owned(),
            Task {
                description: None,
                dependencies: vec!["foo".to_owned()],
                cache: true,
                environment: HashMap::new(),
                input_paths: vec![],
                excluded_input_paths: vec![],
                output_paths: vec![],
                output_paths_on_failure: vec![],
                mount_paths: vec![],
                mount_readonly: false,
                ports: vec![],
                location: None,
                user: None,
                command: String::new(),
                command_prefix: None,
                extra_docker_arguments: vec![],
            },
        );

        let task_file = TaskFile {
            image: "encom:os-12".to_owned(),
            default: None,
            location: UnixPath::new(DEFAULT_LOCATION).to_owned(),
            user: DEFAULT_USER.to_owned(),
            command_prefix: String::new(),
            tasks,
        };

        let result = check_dependencies(&task_file);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cyclic"));
    }

    #[test]
    fn check_dependencies_cycle_2() {
        let mut tasks = HashMap::new();
        tasks.insert(
            "foo".to_owned(),
            Task {
                description: None,
                dependencies: vec!["bar".to_owned()],
                cache: true,
                environment: HashMap::new(),
                input_paths: vec![],
                excluded_input_paths: vec![],
                output_paths: vec![],
                output_paths_on_failure: vec![],
                mount_paths: vec![],
                mount_readonly: false,
                ports: vec![],
                location: None,
                user: None,
                command: String::new(),
                command_prefix: None,
                extra_docker_arguments: vec![],
            },
        );
        tasks.insert(
            "bar".to_owned(),
            Task {
                description: None,
                dependencies: vec!["foo".to_owned()],
                cache: true,
                environment: HashMap::new(),
                input_paths: vec![],
                excluded_input_paths: vec![],
                output_paths: vec![],
                output_paths_on_failure: vec![],
                mount_paths: vec![],
                mount_readonly: false,
                ports: vec![],
                location: None,
                user: None,
                command: String::new(),
                command_prefix: None,
                extra_docker_arguments: vec![],
            },
        );

        let task_file = TaskFile {
            image: "encom:os-12".to_owned(),
            default: None,
            location: UnixPath::new(DEFAULT_LOCATION).to_owned(),
            user: DEFAULT_USER.to_owned(),
            command_prefix: String::new(),
            tasks,
        };

        let result = check_dependencies(&task_file);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cyclic"));
    }

    #[test]
    fn check_dependencies_cycle_3() {
        let mut tasks = HashMap::new();
        tasks.insert(
            "foo".to_owned(),
            Task {
                description: None,
                dependencies: vec!["baz".to_owned()],
                cache: true,
                environment: HashMap::new(),
                input_paths: vec![],
                excluded_input_paths: vec![],
                output_paths: vec![],
                output_paths_on_failure: vec![],
                mount_paths: vec![],
                mount_readonly: false,
                ports: vec![],
                location: None,
                user: None,
                command: String::new(),
                command_prefix: None,
                extra_docker_arguments: vec![],
            },
        );
        tasks.insert(
            "bar".to_owned(),
            Task {
                description: None,
                dependencies: vec!["foo".to_owned()],
                cache: true,
                environment: HashMap::new(),
                input_paths: vec![],
                excluded_input_paths: vec![],
                output_paths: vec![],
                output_paths_on_failure: vec![],
                mount_paths: vec![],
                mount_readonly: false,
                ports: vec![],
                location: None,
                user: None,
                command: String::new(),
                command_prefix: None,
                extra_docker_arguments: vec![],
            },
        );
        tasks.insert(
            "baz".to_owned(),
            Task {
                description: None,
                dependencies: vec!["bar".to_owned()],
                cache: true,
                environment: HashMap::new(),
                input_paths: vec![],
                excluded_input_paths: vec![],
                output_paths: vec![],
                output_paths_on_failure: vec![],
                mount_paths: vec![],
                mount_readonly: false,
                ports: vec![],
                location: None,
                user: None,
                command: String::new(),
                command_prefix: None,
                extra_docker_arguments: vec![],
            },
        );

        let task_file = TaskFile {
            image: "encom:os-12".to_owned(),
            default: None,
            location: UnixPath::new(DEFAULT_LOCATION).to_owned(),
            user: DEFAULT_USER.to_owned(),
            command_prefix: String::new(),
            tasks,
        };

        let result = check_dependencies(&task_file);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cyclic"));
    }

    #[test]
    fn check_task_environment_ok() {
        let mut environment = HashMap::new();
        environment.insert("corge".to_owned(), None);
        environment.insert("grault".to_owned(), None);
        environment.insert("garply".to_owned(), None);

        let task = Task {
            description: None,
            dependencies: vec![],
            cache: true,
            environment,
            input_paths: vec![],
            excluded_input_paths: vec![],
            output_paths: vec![],
            output_paths_on_failure: vec![],
            mount_paths: vec![],
            mount_readonly: false,
            ports: vec![],
            location: None,
            user: None,
            command: String::new(),
            command_prefix: None,
            extra_docker_arguments: vec![],
        };

        assert!(check_task("foo", &task).is_ok());
    }

    #[test]
    fn check_task_environment_equals() {
        let mut environment = HashMap::new();
        environment.insert("corge".to_owned(), None);
        environment.insert("gra=ult".to_owned(), None);
        environment.insert("garply".to_owned(), None);

        let task = Task {
            description: None,
            dependencies: vec![],
            cache: true,
            environment,
            input_paths: vec![],
            excluded_input_paths: vec![],
            output_paths: vec![],
            output_paths_on_failure: vec![],
            mount_paths: vec![],
            mount_readonly: false,
            ports: vec![],
            location: None,
            user: None,
            command: String::new(),
            command_prefix: None,
            extra_docker_arguments: vec![],
        };

        let result = check_task("foo", &task);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains('='));
    }

    #[test]
    fn check_task_paths_ok() {
        let task = Task {
            description: None,
            dependencies: vec![],
            cache: false,
            environment: HashMap::new(),
            input_paths: vec![UnixPath::new("bar").to_owned()],
            excluded_input_paths: vec![UnixPath::new("baz").to_owned()],
            output_paths: vec![UnixPath::new("qux").to_owned()],
            output_paths_on_failure: vec![UnixPath::new("quux").to_owned()],
            mount_paths: vec![
                MappingPath {
                    host_path: Path::new("quuy").to_owned(),
                    container_path: UnixPath::new("quuz").to_owned(),
                },
                MappingPath {
                    host_path: Path::new("quuy").to_owned(),
                    container_path: UnixPath::new("/quuz").to_owned(),
                },
                MappingPath {
                    host_path: Path::new("quuy").to_owned(),
                    container_path: UnixPath::new("/quuz").to_owned(),
                },
                MappingPath {
                    host_path: Path::new("/quuy").to_owned(),
                    container_path: UnixPath::new("/quuz").to_owned(),
                },
            ],
            mount_readonly: false,
            ports: vec![],
            location: Some(UnixPath::new("/corge").to_owned()),
            user: None,
            command: String::new(),
            command_prefix: None,
            extra_docker_arguments: vec![],
        };

        assert!(check_task("foo", &task).is_ok());
    }

    #[test]
    fn check_task_paths_absolute_input_paths() {
        let task = Task {
            description: None,
            dependencies: vec![],
            cache: true,
            environment: HashMap::new(),
            input_paths: vec![UnixPath::new("/bar").to_owned()],
            excluded_input_paths: vec![],
            output_paths: vec![],
            output_paths_on_failure: vec![],
            mount_paths: vec![],
            mount_readonly: false,
            ports: vec![],
            location: None,
            user: None,
            command: String::new(),
            command_prefix: None,
            extra_docker_arguments: vec![],
        };

        let result = check_task("foo", &task);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("/bar"));
    }

    #[test]
    fn check_task_paths_absolute_excluded_input_paths() {
        let task = Task {
            description: None,
            dependencies: vec![],
            cache: true,
            environment: HashMap::new(),
            input_paths: vec![],
            excluded_input_paths: vec![UnixPath::new("/bar").to_owned()],
            output_paths: vec![],
            output_paths_on_failure: vec![],
            mount_paths: vec![],
            mount_readonly: false,
            ports: vec![],
            location: None,
            user: None,
            command: String::new(),
            command_prefix: None,
            extra_docker_arguments: vec![],
        };

        let result = check_task("foo", &task);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("/bar"));
    }

    #[test]
    fn check_task_paths_absolute_output_paths() {
        let task = Task {
            description: None,
            dependencies: vec![],
            cache: false,
            environment: HashMap::new(),
            input_paths: vec![],
            excluded_input_paths: vec![],
            output_paths: vec![UnixPath::new("/bar").to_owned()],
            output_paths_on_failure: vec![],
            mount_paths: vec![],
            mount_readonly: false,
            ports: vec![],
            location: None,
            user: None,
            command: String::new(),
            command_prefix: None,
            extra_docker_arguments: vec![],
        };

        let result = check_task("foo", &task);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("/bar"));
    }

    #[test]
    fn check_task_paths_absolute_output_paths_on_failure() {
        let task = Task {
            description: None,
            dependencies: vec![],
            cache: false,
            environment: HashMap::new(),
            input_paths: vec![],
            excluded_input_paths: vec![],
            output_paths: vec![],
            output_paths_on_failure: vec![UnixPath::new("/bar").to_owned()],
            mount_paths: vec![],
            mount_readonly: false,
            ports: vec![],
            location: None,
            user: None,
            command: String::new(),
            command_prefix: None,
            extra_docker_arguments: vec![],
        };

        let result = check_task("foo", &task);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("/bar"));
    }

    #[test]
    fn check_task_paths_mount_paths_comma() {
        let task = Task {
            description: None,
            dependencies: vec![],
            cache: true,
            environment: HashMap::new(),
            input_paths: vec![],
            excluded_input_paths: vec![],
            output_paths: vec![],
            output_paths_on_failure: vec![],
            mount_paths: vec![MappingPath {
                host_path: Path::new("bar,baz").to_owned(),
                container_path: UnixPath::new("bar,baz").to_owned(),
            }],
            mount_readonly: false,
            ports: vec![],
            location: None,
            user: None,
            command: String::new(),
            command_prefix: None,
            extra_docker_arguments: vec![],
        };

        let result = check_task("foo", &task);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("bar,baz"));
    }

    #[test]
    fn check_task_paths_relative_location() {
        let task = Task {
            description: None,
            dependencies: vec![],
            cache: true,
            environment: HashMap::new(),
            input_paths: vec![],
            excluded_input_paths: vec![],
            output_paths: vec![],
            output_paths_on_failure: vec![],
            mount_paths: vec![],
            mount_readonly: false,
            ports: vec![],
            location: Some(UnixPath::new("code").to_owned()),
            user: None,
            command: String::new(),
            command_prefix: None,
            extra_docker_arguments: vec![],
        };

        let result = check_task("foo", &task);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("code"));
    }

    #[test]
    fn check_task_caching_enabled_with_mount_paths() {
        let task = Task {
            description: None,
            dependencies: vec![],
            cache: true,
            environment: HashMap::new(),
            input_paths: vec![],
            excluded_input_paths: vec![],
            output_paths: vec![],
            output_paths_on_failure: vec![],
            mount_paths: vec![MappingPath {
                host_path: Path::new("bar").to_owned(),
                container_path: UnixPath::new("bar").to_owned(),
            }],
            mount_readonly: false,
            ports: vec![],
            location: None,
            user: None,
            command: String::new(),
            command_prefix: None,
            extra_docker_arguments: vec![],
        };

        let result = check_task("foo", &task);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("mount_paths"));
    }

    #[test]
    fn check_task_caching_disabled_with_mount_paths() {
        let task = Task {
            description: None,
            dependencies: vec![],
            cache: false,
            environment: HashMap::new(),
            input_paths: vec![],
            excluded_input_paths: vec![],
            output_paths: vec![],
            output_paths_on_failure: vec![],
            mount_paths: vec![MappingPath {
                host_path: Path::new("bar").to_owned(),
                container_path: UnixPath::new("bar").to_owned(),
            }],
            mount_readonly: false,
            ports: vec!["3000:80".to_owned()],
            location: None,
            user: None,
            command: String::new(),
            command_prefix: None,
            extra_docker_arguments: vec![],
        };

        assert!(check_task("foo", &task).is_ok());
    }

    #[test]
    fn check_task_caching_enabled_with_ports() {
        let task = Task {
            description: None,
            dependencies: vec![],
            cache: true,
            environment: HashMap::new(),
            input_paths: vec![],
            excluded_input_paths: vec![],
            output_paths: vec![],
            output_paths_on_failure: vec![],
            mount_paths: vec![],
            mount_readonly: false,
            ports: vec!["3000:80".to_owned()],
            location: None,
            user: None,
            command: String::new(),
            command_prefix: None,
            extra_docker_arguments: vec![],
        };

        let result = check_task("foo", &task);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("caching"));
    }

    #[test]
    fn check_task_caching_disabled_with_ports() {
        let task = Task {
            description: None,
            dependencies: vec![],
            cache: false,
            environment: HashMap::new(),
            input_paths: vec![],
            excluded_input_paths: vec![],
            output_paths: vec![],
            output_paths_on_failure: vec![],
            mount_paths: vec![],
            mount_readonly: false,
            ports: vec!["3000:80".to_owned()],
            location: None,
            user: None,
            command: String::new(),
            command_prefix: None,
            extra_docker_arguments: vec![],
        };

        assert!(check_task("foo", &task).is_ok());
    }

    #[test]
    fn check_task_caching_enabled_with_extra_docker_arguments() {
        let task = Task {
            description: None,
            dependencies: vec![],
            cache: true,
            environment: HashMap::new(),
            input_paths: vec![],
            excluded_input_paths: vec![],
            output_paths: vec![],
            output_paths_on_failure: vec![],
            mount_paths: vec![],
            mount_readonly: false,
            ports: vec![],
            location: None,
            user: None,
            command: String::new(),
            command_prefix: None,
            extra_docker_arguments: vec!["--cpus".to_owned(), "4".to_owned()],
        };

        let result = check_task("foo", &task);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("caching"));
    }

    #[test]
    fn check_task_caching_disabled_with_extra_docker_arguments() {
        let task = Task {
            description: None,
            dependencies: vec![],
            cache: false,
            environment: HashMap::new(),
            input_paths: vec![],
            excluded_input_paths: vec![],
            output_paths: vec![],
            output_paths_on_failure: vec![],
            mount_paths: vec![],
            mount_readonly: false,
            ports: vec![],
            location: None,
            user: None,
            command: String::new(),
            command_prefix: None,
            extra_docker_arguments: vec!["--cpus".to_owned(), "4".to_owned()],
        };

        assert!(check_task("foo", &task).is_ok());
    }

    #[test]
    fn environment_empty() {
        let task = Task {
            description: None,
            dependencies: vec![],
            cache: true,
            environment: HashMap::new(),
            input_paths: vec![],
            excluded_input_paths: vec![],
            output_paths: vec![],
            output_paths_on_failure: vec![],
            mount_paths: vec![],
            mount_readonly: false,
            ports: vec![],
            location: None,
            user: None,
            command: String::new(),
            command_prefix: None,
            extra_docker_arguments: vec![],
        };

        assert_eq!(environment(&task), Ok(HashMap::new()));
    }

    #[test]
    fn environment_default_overridden() {
        // NOTE: We add an index to the test arg ("foo1", "foo2", ...) to avoid having parallel
        // tests clobbering environment variables used by other threads.
        let mut env_map = HashMap::new();
        env_map.insert("foo1".to_owned(), Some("bar".to_owned()));

        let task = Task {
            description: None,
            dependencies: vec![],
            cache: true,
            environment: env_map,
            input_paths: vec![],
            excluded_input_paths: vec![],
            output_paths: vec![],
            output_paths_on_failure: vec![],
            mount_paths: vec![],
            mount_readonly: false,
            ports: vec![],
            location: None,
            user: None,
            command: String::new(),
            command_prefix: None,
            extra_docker_arguments: vec![],
        };

        let mut expected = HashMap::new();
        expected.insert("foo1".to_owned(), "baz".to_owned());

        env::set_var("foo1", "baz");
        assert_eq!(env::var("foo1"), Ok("baz".to_owned()));
        assert_eq!(environment(&task), Ok(expected));
    }

    #[test]
    fn environment_default_not_overridden() {
        // NOTE: We add an index to the test arg ("foo1", "foo2", ...) to avoid having parallel
        // tests clobbering environment variables used by other threads.
        let mut env_map = HashMap::new();
        env_map.insert("foo2".to_owned(), Some("bar".to_owned()));

        let task = Task {
            description: None,
            dependencies: vec![],
            cache: true,
            environment: env_map,
            input_paths: vec![],
            excluded_input_paths: vec![],
            output_paths: vec![],
            output_paths_on_failure: vec![],
            mount_paths: vec![],
            mount_readonly: false,
            ports: vec![],
            location: None,
            user: None,
            command: String::new(),
            command_prefix: None,
            extra_docker_arguments: vec![],
        };

        let mut expected = HashMap::new();
        expected.insert("foo2".to_owned(), "bar".to_owned());

        env::remove_var("foo2");
        assert!(env::var("foo2").is_err());
        assert_eq!(environment(&task), Ok(expected));
    }

    #[test]
    fn environment_missing() {
        // NOTE: We add an index to the test arg ("foo1", "foo2", ...) to avoid having parallel
        // tests clobbering environment variables used by other threads.
        let mut env_map = HashMap::new();
        env_map.insert("foo3".to_owned(), None);

        let task = Task {
            description: None,
            dependencies: vec![],
            cache: true,
            environment: env_map,
            input_paths: vec![],
            excluded_input_paths: vec![],
            output_paths: vec![],
            output_paths_on_failure: vec![],
            mount_paths: vec![],
            mount_readonly: false,
            ports: vec![],
            location: None,
            user: None,
            command: String::new(),
            command_prefix: None,
            extra_docker_arguments: vec![],
        };

        env::remove_var("foo3");
        assert!(env::var("foo3").is_err());
        let result = environment(&task);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err()[0].to_owned(), "foo3");
    }

    #[test]
    fn location_default() {
        let mut tasks = HashMap::new();
        tasks.insert(
            "foo".to_owned(),
            Task {
                description: None,
                dependencies: vec![],
                cache: true,
                environment: HashMap::new(),
                input_paths: vec![],
                excluded_input_paths: vec![],
                output_paths: vec![],
                output_paths_on_failure: vec![],
                mount_paths: vec![],
                mount_readonly: false,
                ports: vec![],
                location: None,
                user: None,
                command: String::new(),
                command_prefix: None,
                extra_docker_arguments: vec![],
            },
        );

        let task_file = TaskFile {
            image: "encom:os-12".to_owned(),
            default: None,
            location: UnixPath::new(DEFAULT_LOCATION).to_owned(),
            user: DEFAULT_USER.to_owned(),
            command_prefix: String::new(),
            tasks,
        };

        assert_eq!(
            location(&task_file, &task_file.tasks["foo"]),
            UnixPath::new(DEFAULT_LOCATION),
        );
    }

    #[test]
    fn location_override() {
        let mut tasks = HashMap::new();
        tasks.insert(
            "foo".to_owned(),
            Task {
                description: None,
                dependencies: vec![],
                cache: true,
                environment: HashMap::new(),
                input_paths: vec![],
                excluded_input_paths: vec![],
                output_paths: vec![],
                output_paths_on_failure: vec![],
                mount_paths: vec![],
                mount_readonly: false,
                ports: vec![],
                location: Some(UnixPath::new("/bar").to_owned()),
                user: None,
                command: String::new(),
                command_prefix: None,
                extra_docker_arguments: vec![],
            },
        );

        let task_file = TaskFile {
            image: "encom:os-12".to_owned(),
            default: None,
            location: UnixPath::new(DEFAULT_LOCATION).to_owned(),
            user: DEFAULT_USER.to_owned(),
            command_prefix: String::new(),
            tasks,
        };

        assert_eq!(
            location(&task_file, &task_file.tasks["foo"]),
            UnixPath::new("/bar"),
        );
    }

    #[test]
    fn user_default() {
        let mut tasks = HashMap::new();
        tasks.insert(
            "foo".to_owned(),
            Task {
                description: None,
                dependencies: vec![],
                cache: true,
                environment: HashMap::new(),
                input_paths: vec![],
                excluded_input_paths: vec![],
                output_paths: vec![],
                output_paths_on_failure: vec![],
                mount_paths: vec![],
                mount_readonly: false,
                ports: vec![],
                location: None,
                user: None,
                command: String::new(),
                command_prefix: None,
                extra_docker_arguments: vec![],
            },
        );

        let task_file = TaskFile {
            image: "encom:os-12".to_owned(),
            default: None,
            location: UnixPath::new(DEFAULT_LOCATION).to_owned(),
            user: DEFAULT_USER.to_owned(),
            command_prefix: String::new(),
            tasks,
        };

        assert_eq!(
            user(&task_file, &task_file.tasks["foo"]),
            DEFAULT_USER.to_owned(),
        );
    }

    #[test]
    fn user_override() {
        let mut tasks = HashMap::new();
        tasks.insert(
            "foo".to_owned(),
            Task {
                description: None,
                dependencies: vec![],
                cache: true,
                environment: HashMap::new(),
                input_paths: vec![],
                excluded_input_paths: vec![],
                output_paths: vec![],
                output_paths_on_failure: vec![],
                mount_paths: vec![],
                mount_readonly: false,
                ports: vec![],
                location: None,
                user: Some("bar".to_owned()),
                command: String::new(),
                command_prefix: None,
                extra_docker_arguments: vec![],
            },
        );

        let task_file = TaskFile {
            image: "encom:os-12".to_owned(),
            default: None,
            location: UnixPath::new(DEFAULT_LOCATION).to_owned(),
            user: DEFAULT_USER.to_owned(),
            command_prefix: String::new(),
            tasks,
        };

        assert_eq!(user(&task_file, &task_file.tasks["foo"]), "bar");
    }

    #[test]
    fn command_default_prefix_default() {
        let mut tasks = HashMap::new();
        tasks.insert(
            "foo".to_owned(),
            Task {
                description: None,
                dependencies: vec![],
                cache: true,
                environment: HashMap::new(),
                input_paths: vec![],
                excluded_input_paths: vec![],
                output_paths: vec![],
                output_paths_on_failure: vec![],
                mount_paths: vec![],
                mount_readonly: false,
                ports: vec![],
                location: None,
                user: None,
                command: String::new(),
                command_prefix: None,
                extra_docker_arguments: vec![],
            },
        );

        let task_file = TaskFile {
            image: "encom:os-12".to_owned(),
            default: None,
            location: UnixPath::new(DEFAULT_LOCATION).to_owned(),
            user: DEFAULT_USER.to_owned(),
            command_prefix: "set -euxo pipefail".to_owned(),
            tasks,
        };

        assert_eq!(
            command(&task_file, &task_file.tasks["foo"]),
            "set -euxo pipefail".to_owned(),
        );
    }

    #[test]
    fn command_override_prefix_default() {
        let mut tasks = HashMap::new();
        tasks.insert(
            "foo".to_owned(),
            Task {
                description: None,
                dependencies: vec![],
                cache: true,
                environment: HashMap::new(),
                input_paths: vec![],
                excluded_input_paths: vec![],
                output_paths: vec![],
                output_paths_on_failure: vec![],
                mount_paths: vec![],
                mount_readonly: false,
                ports: vec![],
                location: None,
                user: None,
                command: "echo hello".to_owned(),
                command_prefix: None,
                extra_docker_arguments: vec![],
            },
        );

        let task_file = TaskFile {
            image: "encom:os-12".to_owned(),
            default: None,
            location: UnixPath::new(DEFAULT_LOCATION).to_owned(),
            user: DEFAULT_USER.to_owned(),
            command_prefix: String::new(),
            tasks,
        };

        assert_eq!(
            command(&task_file, &task_file.tasks["foo"]),
            "echo hello".to_owned(),
        );
    }

    #[test]
    fn command_default_prefix_override() {
        let mut tasks = HashMap::new();
        tasks.insert(
            "foo".to_owned(),
            Task {
                description: None,
                dependencies: vec![],
                cache: true,
                environment: HashMap::new(),
                input_paths: vec![],
                excluded_input_paths: vec![],
                output_paths: vec![],
                output_paths_on_failure: vec![],
                mount_paths: vec![],
                mount_readonly: false,
                ports: vec![],
                location: None,
                user: None,
                command: String::new(),
                command_prefix: Some("set -euxo pipefail".to_owned()),
                extra_docker_arguments: vec![],
            },
        );

        let task_file = TaskFile {
            image: "encom:os-12".to_owned(),
            default: None,
            location: UnixPath::new(DEFAULT_LOCATION).to_owned(),
            user: DEFAULT_USER.to_owned(),
            command_prefix: String::new(),
            tasks,
        };

        assert_eq!(
            command(&task_file, &task_file.tasks["foo"]),
            "set -euxo pipefail".to_owned(),
        );
    }

    #[test]
    fn command_override_prefix_override() {
        let mut tasks = HashMap::new();
        tasks.insert(
            "foo".to_owned(),
            Task {
                description: None,
                dependencies: vec![],
                cache: true,
                environment: HashMap::new(),
                input_paths: vec![],
                excluded_input_paths: vec![],
                output_paths: vec![],
                output_paths_on_failure: vec![],
                mount_paths: vec![],
                mount_readonly: false,
                ports: vec![],
                location: None,
                user: None,
                command: "echo hello".to_owned(),
                command_prefix: Some("set -euxo pipefail".to_owned()),
                extra_docker_arguments: vec![],
            },
        );

        let task_file = TaskFile {
            image: "encom:os-12".to_owned(),
            default: None,
            location: UnixPath::new(DEFAULT_LOCATION).to_owned(),
            user: DEFAULT_USER.to_owned(),
            command_prefix: String::new(),
            tasks,
        };

        assert_eq!(
            command(&task_file, &task_file.tasks["foo"]),
            "set -euxo pipefail\necho hello".to_owned(),
        );
    }

    fn taskfile_with_task(foo_task: Task) -> TaskFile {
        let mut tasks = HashMap::new();
        tasks.insert("foo".to_owned(), foo_task);

        TaskFile {
            image: "encom:os-12".to_owned(),
            default: None,
            location: UnixPath::new(DEFAULT_LOCATION).to_owned(),
            user: DEFAULT_USER.to_owned(),
            command_prefix: String::new(),
            tasks,
        }
    }

    fn taskfile_with_tasks(foo_task: Task, bar_task: Task) -> TaskFile {
        let mut tasks = HashMap::new();
        tasks.insert("foo".to_owned(), foo_task);
        tasks.insert("bar".to_owned(), bar_task);

        TaskFile {
            image: "encom:os-12".to_owned(),
            default: None,
            location: UnixPath::new(DEFAULT_LOCATION).to_owned(),
            user: DEFAULT_USER.to_owned(),
            command_prefix: String::new(),
            tasks,
        }
    }

    #[test]
    fn image_name_noop() {
        let previous_image = "corge";
        let docker_repo = "task";

        let environment: HashMap<String, Option<String>> = HashMap::new();

        let task = Task {
            description: None,
            dependencies: vec![],
            cache: true,
            environment,
            input_paths: vec![],
            excluded_input_paths: vec![],
            output_paths: vec![],
            output_paths_on_failure: vec![],
            mount_paths: vec![],
            mount_readonly: false,
            ports: vec![],
            location: None,
            user: None,
            command: String::new(),
            command_prefix: None,
            extra_docker_arguments: vec![],
        };

        let taskfile = taskfile_with_task(task);

        let input_files_hash = "grault";

        let full_environment = HashMap::new();

        assert_eq!(
            previous_image,
            image_name(
                previous_image,
                docker_repo,
                &taskfile,
                &taskfile.tasks["foo"],
                input_files_hash,
                &full_environment,
            ),
        );
    }

    #[test]
    fn image_name_pure() {
        let previous_image = "corge";
        let docker_repo = "task";

        let mut environment: HashMap<String, Option<String>> = HashMap::new();
        environment.insert("foo".to_owned(), None);

        let task = Task {
            description: None,
            dependencies: vec![],
            cache: true,
            environment,
            input_paths: vec![UnixPath::new("flob").to_owned()],
            excluded_input_paths: vec![UnixPath::new("thud").to_owned()],
            output_paths: vec![],
            output_paths_on_failure: vec![],
            mount_paths: vec![],
            mount_readonly: false,
            ports: vec![],
            location: None,
            user: None,
            command: "echo wibble".to_owned(),
            command_prefix: None,
            extra_docker_arguments: vec![],
        };

        let taskfile = taskfile_with_task(task);

        let input_files_hash = "grault";

        let mut full_environment = HashMap::new();
        full_environment.insert("foo".to_owned(), "qux".to_owned());

        assert_eq!(
            image_name(
                previous_image,
                docker_repo,
                &taskfile,
                &taskfile.tasks["foo"],
                input_files_hash,
                &full_environment,
            ),
            image_name(
                previous_image,
                docker_repo,
                &taskfile,
                &taskfile.tasks["foo"],
                input_files_hash,
                &full_environment,
            ),
        );
    }

    #[test]
    fn image_name_previous_image() {
        let previous_image1 = "foo";
        let previous_image2 = "bar";
        let docker_repo = "task";

        let task = Task {
            description: None,
            dependencies: vec![],
            cache: true,
            environment: HashMap::new(),
            input_paths: vec![],
            excluded_input_paths: vec![],
            output_paths: vec![],
            output_paths_on_failure: vec![],
            mount_paths: vec![],
            mount_readonly: false,
            ports: vec![],
            location: None,
            user: None,
            command: "echo wibble".to_owned(),
            command_prefix: None,
            extra_docker_arguments: vec![],
        };

        let taskfile = taskfile_with_task(task);

        let input_files_hash = "grault";

        let full_environment = HashMap::new();

        assert_ne!(
            image_name(
                previous_image1,
                docker_repo,
                &taskfile,
                &taskfile.tasks["foo"],
                input_files_hash,
                &full_environment,
            ),
            image_name(
                previous_image2,
                docker_repo,
                &taskfile,
                &taskfile.tasks["foo"],
                input_files_hash,
                &full_environment,
            ),
        );
    }

    #[test]
    fn image_name_environment_order() {
        let previous_image = "corge";
        let docker_repo = "task";

        let mut environment1: HashMap<String, Option<String>> = HashMap::new();
        environment1.insert("foo".to_owned(), None);
        environment1.insert("bar".to_owned(), None);

        let mut environment2: HashMap<String, Option<String>> = HashMap::new();
        environment2.insert("bar".to_owned(), None);
        environment2.insert("foo".to_owned(), None);

        let task1 = Task {
            description: None,
            dependencies: vec![],
            cache: true,
            environment: environment1,
            input_paths: vec![],
            excluded_input_paths: vec![],
            output_paths: vec![],
            output_paths_on_failure: vec![],
            mount_paths: vec![],
            mount_readonly: false,
            ports: vec![],
            location: None,
            user: None,
            command: "echo wibble".to_owned(),
            command_prefix: None,
            extra_docker_arguments: vec![],
        };

        let task2 = Task {
            description: None,
            dependencies: vec![],
            cache: true,
            environment: environment2,
            input_paths: vec![],
            excluded_input_paths: vec![],
            output_paths: vec![],
            output_paths_on_failure: vec![],
            mount_paths: vec![],
            mount_readonly: false,
            ports: vec![],
            location: None,
            user: None,
            command: "echo wibble".to_owned(),
            command_prefix: None,
            extra_docker_arguments: vec![],
        };

        let taskfile = taskfile_with_tasks(task1, task2);

        let input_files_hash = "grault";

        let mut full_environment = HashMap::new();
        full_environment.insert("foo".to_owned(), "qux".to_owned());
        full_environment.insert("bar".to_owned(), "fum".to_owned());

        assert_eq!(
            image_name(
                previous_image,
                docker_repo,
                &taskfile,
                &taskfile.tasks["foo"],
                input_files_hash,
                &full_environment,
            ),
            image_name(
                previous_image,
                docker_repo,
                &taskfile,
                &taskfile.tasks["bar"],
                input_files_hash,
                &full_environment,
            ),
        );
    }

    #[test]
    fn image_name_environment_keys() {
        let previous_image = "corge";
        let docker_repo = "task";

        let mut environment1: HashMap<String, Option<String>> = HashMap::new();
        environment1.insert("foo".to_owned(), None);

        let mut environment2: HashMap<String, Option<String>> = HashMap::new();
        environment2.insert("bar".to_owned(), None);

        let task1 = Task {
            description: None,
            dependencies: vec![],
            cache: true,
            environment: environment1,
            input_paths: vec![],
            excluded_input_paths: vec![],
            output_paths: vec![],
            output_paths_on_failure: vec![],
            mount_paths: vec![],
            mount_readonly: false,
            ports: vec![],
            location: None,
            user: None,
            command: "echo wibble".to_owned(),
            command_prefix: None,
            extra_docker_arguments: vec![],
        };

        let task2 = Task {
            description: None,
            dependencies: vec![],
            cache: true,
            environment: environment2,
            input_paths: vec![],
            excluded_input_paths: vec![],
            output_paths: vec![],
            output_paths_on_failure: vec![],
            mount_paths: vec![],
            mount_readonly: false,
            ports: vec![],
            location: None,
            user: None,
            command: "echo wibble".to_owned(),
            command_prefix: None,
            extra_docker_arguments: vec![],
        };

        let taskfile = taskfile_with_tasks(task1, task2);

        let input_files_hash = "grault";

        let mut full_environment = HashMap::new();
        full_environment.insert("foo".to_owned(), "qux".to_owned());
        full_environment.insert("bar".to_owned(), "fum".to_owned());

        assert_ne!(
            image_name(
                previous_image,
                docker_repo,
                &taskfile,
                &taskfile.tasks["foo"],
                input_files_hash,
                &full_environment,
            ),
            image_name(
                previous_image,
                docker_repo,
                &taskfile,
                &taskfile.tasks["bar"],
                input_files_hash,
                &full_environment,
            ),
        );
    }

    #[test]
    fn image_name_environment_values() {
        let previous_image = "corge";
        let docker_repo = "task";

        let mut environment: HashMap<String, Option<String>> = HashMap::new();
        environment.insert("foo".to_owned(), None);

        let task = Task {
            description: None,
            dependencies: vec![],
            cache: true,
            environment,
            input_paths: vec![],
            excluded_input_paths: vec![],
            output_paths: vec![],
            output_paths_on_failure: vec![],
            mount_paths: vec![],
            mount_readonly: false,
            ports: vec![],
            location: None,
            user: None,
            command: "echo wibble".to_owned(),
            command_prefix: None,
            extra_docker_arguments: vec![],
        };

        let taskfile = taskfile_with_task(task);

        let input_files_hash = "grault";

        let mut full_environment1 = HashMap::new();
        full_environment1.insert("foo".to_owned(), "bar".to_owned());
        let mut full_environment2 = HashMap::new();
        full_environment2.insert("foo".to_owned(), "baz".to_owned());

        assert_ne!(
            image_name(
                previous_image,
                docker_repo,
                &taskfile,
                &taskfile.tasks["foo"],
                input_files_hash,
                &full_environment1,
            ),
            image_name(
                previous_image,
                docker_repo,
                &taskfile,
                &taskfile.tasks["foo"],
                input_files_hash,
                &full_environment2,
            ),
        );
    }

    #[test]
    fn image_name_input_files_hash() {
        let previous_image = "corge";
        let docker_repo = "task";

        let task = Task {
            description: None,
            dependencies: vec![],
            cache: true,
            environment: HashMap::new(),
            input_paths: vec![UnixPath::new("flob").to_owned()],
            excluded_input_paths: vec![UnixPath::new("thud").to_owned()],
            output_paths: vec![],
            output_paths_on_failure: vec![],
            mount_paths: vec![],
            mount_readonly: false,
            ports: vec![],
            location: None,
            user: None,
            command: "echo wibble".to_owned(),
            command_prefix: None,
            extra_docker_arguments: vec![],
        };

        let taskfile = taskfile_with_task(task);

        let input_files_hash1 = "foo";
        let input_files_hash2 = "bar";

        let full_environment = HashMap::new();

        assert_ne!(
            image_name(
                previous_image,
                docker_repo,
                &taskfile,
                &taskfile.tasks["foo"],
                input_files_hash1,
                &full_environment,
            ),
            image_name(
                previous_image,
                docker_repo,
                &taskfile,
                &taskfile.tasks["foo"],
                input_files_hash2,
                &full_environment,
            ),
        );
    }

    #[test]
    fn image_name_location() {
        let previous_image = "corge";
        let docker_repo = "task";

        let task1 = Task {
            description: None,
            dependencies: vec![],
            cache: true,
            environment: HashMap::new(),
            input_paths: vec![],
            excluded_input_paths: vec![],
            output_paths: vec![],
            output_paths_on_failure: vec![],
            mount_paths: vec![],
            mount_readonly: false,
            ports: vec![],
            location: Some(UnixPath::new("/foo").to_owned()),
            user: None,
            command: "echo wibble".to_owned(),
            command_prefix: None,
            extra_docker_arguments: vec![],
        };

        let task2 = Task {
            description: None,
            dependencies: vec![],
            cache: true,
            environment: HashMap::new(),
            input_paths: vec![],
            excluded_input_paths: vec![],
            output_paths: vec![],
            output_paths_on_failure: vec![],
            mount_paths: vec![],
            mount_readonly: false,
            ports: vec![],
            location: Some(UnixPath::new("/bar").to_owned()),
            user: None,
            command: "echo wibble".to_owned(),
            command_prefix: None,
            extra_docker_arguments: vec![],
        };

        let taskfile = taskfile_with_tasks(task1, task2);

        let input_files_hash = "grault";

        let full_environment = HashMap::new();

        assert_ne!(
            image_name(
                previous_image,
                docker_repo,
                &taskfile,
                &taskfile.tasks["foo"],
                input_files_hash,
                &full_environment,
            ),
            image_name(
                previous_image,
                docker_repo,
                &taskfile,
                &taskfile.tasks["bar"],
                input_files_hash,
                &full_environment,
            ),
        );
    }

    #[test]
    fn image_name_user() {
        let previous_image = "corge";
        let docker_repo = "task";

        let task1 = Task {
            description: None,
            dependencies: vec![],
            cache: true,
            environment: HashMap::new(),
            input_paths: vec![],
            excluded_input_paths: vec![],
            output_paths: vec![],
            output_paths_on_failure: vec![],
            mount_paths: vec![],
            mount_readonly: false,
            ports: vec![],
            location: None,
            user: Some("foo".to_owned()),
            command: "echo wibble".to_owned(),
            command_prefix: None,
            extra_docker_arguments: vec![],
        };

        let task2 = Task {
            description: None,
            dependencies: vec![],
            cache: true,
            environment: HashMap::new(),
            input_paths: vec![],
            excluded_input_paths: vec![],
            output_paths: vec![],
            output_paths_on_failure: vec![],
            mount_paths: vec![],
            mount_readonly: false,
            ports: vec![],
            location: None,
            user: Some("bar".to_owned()),
            command: "echo wibble".to_owned(),
            command_prefix: None,
            extra_docker_arguments: vec![],
        };

        let taskfile = taskfile_with_tasks(task1, task2);

        let input_files_hash = "grault";

        let full_environment = HashMap::new();

        assert_ne!(
            image_name(
                previous_image,
                docker_repo,
                &taskfile,
                &taskfile.tasks["foo"],
                input_files_hash,
                &full_environment,
            ),
            image_name(
                previous_image,
                docker_repo,
                &taskfile,
                &taskfile.tasks["bar"],
                input_files_hash,
                &full_environment,
            ),
        );
    }

    #[test]
    fn image_name_command() {
        let previous_image = "corge";
        let docker_repo = "task";

        let task1 = Task {
            description: None,
            dependencies: vec![],
            cache: true,
            environment: HashMap::new(),
            input_paths: vec![],
            excluded_input_paths: vec![],
            output_paths: vec![],
            output_paths_on_failure: vec![],
            mount_paths: vec![],
            mount_readonly: false,
            ports: vec![],
            location: None,
            user: None,
            command: "echo foo".to_owned(),
            command_prefix: None,
            extra_docker_arguments: vec![],
        };

        let task2 = Task {
            description: None,
            dependencies: vec![],
            cache: true,
            environment: HashMap::new(),
            input_paths: vec![],
            excluded_input_paths: vec![],
            output_paths: vec![],
            output_paths_on_failure: vec![],
            mount_paths: vec![],
            mount_readonly: false,
            ports: vec![],
            location: None,
            user: None,
            command: "echo bar".to_owned(),
            command_prefix: None,
            extra_docker_arguments: vec![],
        };

        let taskfile = taskfile_with_tasks(task1, task2);

        let input_files_hash = "grault";

        let full_environment = HashMap::new();

        assert_ne!(
            image_name(
                previous_image,
                docker_repo,
                &taskfile,
                &taskfile.tasks["foo"],
                input_files_hash,
                &full_environment,
            ),
            image_name(
                previous_image,
                docker_repo,
                &taskfile,
                &taskfile.tasks["bar"],
                input_files_hash,
                &full_environment,
            ),
        );
    }
}
