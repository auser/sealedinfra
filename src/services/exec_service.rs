use std::{
    process::{ChildStdin, Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use crate::{
    error::{SealedError, SealedResult},
    util::spinner::spin,
};

// Run a command and return its standard output.
pub fn run_quiet(
    docker_cli: &str,
    spinner_message: &str,
    error: &str,
    args: &[String],
    user_command: bool,
    interrupted: &Arc<AtomicBool>,
) -> SealedResult<String> {
    // Render a spinner animation and clear it when we're done.
    let _guard = spin(spinner_message);

    // This is used to determine whether the user interrupted the program during the execution of
    // the child process.
    let was_interrupted = interrupted.load(Ordering::SeqCst);

    // Run the child process.
    let child = command(docker_cli, args).output().map_err(|error| {
        SealedError::System(
            format!("{error} Perhaps you don't have Docker installed.",),
            None,
        )
    })?;

    // Handle the result.
    if child.status.success() {
        Ok(String::from_utf8_lossy(&child.stdout).to_string())
    } else {
        Err(
            if child.status.code().is_none()
                || (!was_interrupted && interrupted.load(Ordering::SeqCst))
            {
                interrupted.store(true, Ordering::SeqCst);
                SealedError::Interrupted
            } else if user_command {
                SealedError::FailedToRunUserCommand(
                    format!("{}\n{}", error, String::from_utf8_lossy(&child.stderr)),
                    None,
                )
            } else {
                SealedError::System(
                    format!("{}\n{}", error, String::from_utf8_lossy(&child.stderr)),
                    None,
                )
            },
        )
    }
}

// Run a command and return its standard output. Accepts a closure which receives a pipe to the
// standard input stream of the child process.
pub fn run_quiet_stdin<W: FnOnce(&mut ChildStdin) -> SealedResult<()>>(
    docker_cli: &str,
    spinner_message: &str,
    error: &str,
    args: &[String],
    user_command: bool,
    writer: W,
    interrupted: &Arc<AtomicBool>,
) -> SealedResult<String> {
    // Render a spinner animation and clear it when we're done.
    let _guard = spin(spinner_message);

    // This is used to determine whether the user interrupted the program during the execution of
    // the child process.
    let was_interrupted = interrupted.load(Ordering::SeqCst);

    // Run the child process.
    let mut child = command(docker_cli, args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            SealedError::System(
                format!("{error} Perhaps you don't have Docker installed.",),
                None,
            )
        })?;

    // Pipe data to the child's standard input stream.
    writer(child.stdin.as_mut().unwrap())?; // [ref:run_quiet_stdin_piped]

    // Wait for the child to terminate.
    let output = child.wait_with_output().map_err(|error| {
        SealedError::System(
            format!("{error} Perhaps you don't have Docker installed.",),
            None,
        )
    })?;

    // Handle the result.
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(
            if output.status.code().is_none()
                || (!was_interrupted && interrupted.load(Ordering::SeqCst))
            {
                interrupted.store(true, Ordering::SeqCst);
                SealedError::Interrupted
            } else if user_command {
                SealedError::FailedToRunUserCommand(
                    format!("{}\n{}", error, String::from_utf8_lossy(&output.stderr)),
                    None,
                )
            } else {
                SealedError::System(
                    format!("{}\n{}", error, String::from_utf8_lossy(&output.stderr)),
                    None,
                )
            },
        )
    }
}

// Run a command and inherit standard output and error streams.
pub fn run_loud(
    docker_cli: &str,
    error: &str,
    args: &[String],
    user_command: bool,
    interrupted: &Arc<AtomicBool>,
) -> SealedResult<()> {
    // This is used to determine whether the user interrupted the program during the execution of
    // the child process.
    let was_interrupted = interrupted.load(Ordering::SeqCst);

    // Run the child process.
    let mut child = command(docker_cli, args)
        .stdin(Stdio::null())
        .spawn()
        .map_err(|error| {
            SealedError::System(
                format!("{error} Perhaps you don't have Docker installed."),
                None,
            )
        })?;
    // Wait for the child to terminate.
    let status = child.wait().map_err(|error| {
        SealedError::System(
            format!("{error} Perhaps you don't have Docker installed."),
            None,
        )
    })?;

    // Handle the result.
    if status.success() {
        Ok(())
    } else {
        Err(
            if status.code().is_none() || (!was_interrupted && interrupted.load(Ordering::SeqCst)) {
                interrupted.store(true, Ordering::SeqCst);
                SealedError::Interrupted
            } else if user_command {
                SealedError::FailedToRunUserCommand(error.to_owned(), None)
            } else {
                SealedError::System(error.to_owned(), None)
            },
        )
    }
}

// Run a command and inherit standard input, output, and error streams.
pub fn run_attach(
    docker_cli: &str,
    error: &str,
    args: &[String],
    user_command: bool,
    interrupted: &Arc<AtomicBool>,
) -> SealedResult<()> {
    // This is used to determine whether the user interrupted the program during the execution of
    // the child process.
    let was_interrupted = interrupted.load(Ordering::SeqCst);

    // Run the child process.
    let child = command(docker_cli, args).status().map_err(|error| {
        SealedError::System(
            format!("{error} Perhaps you don't have Docker installed."),
            None,
        )
    })?;

    // Handle the result.
    if child.success() {
        Ok(())
    } else {
        Err(
            if child.code().is_none() || (!was_interrupted && interrupted.load(Ordering::SeqCst)) {
                interrupted.store(true, Ordering::SeqCst);
                SealedError::Interrupted
            } else if user_command {
                SealedError::FailedToRunUserCommand(error.to_owned(), None)
            } else {
                SealedError::System(error.to_owned(), None)
            },
        )
    }
}

// Construct a Docker `Command` from an array of arguments.
pub fn command(docker_cli: &str, args: &[String]) -> Command {
    let mut command = Command::new(docker_cli);
    for arg in args {
        command.arg(arg);
    }
    command
}
