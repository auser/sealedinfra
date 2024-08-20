use log::info;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

pub async fn stream_command_output(
    command: &str,
    args: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(command);
    cmd.args(args);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn()?;

    let stdout = child
        .stdout
        .take()
        .expect("Child process should have a handle to stdout");
    let stderr = child
        .stderr
        .take()
        .expect("Child process should have a handle to stderr");

    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    let stdout_handle = tokio::spawn(async move {
        while let Some(line) = stdout_reader
            .next_line()
            .await
            .expect("Failed to read line")
        {
            info!("{}", line);
        }
    });

    let stderr_handle = tokio::spawn(async move {
        while let Some(line) = stderr_reader
            .next_line()
            .await
            .expect("Failed to read line")
        {
            info!("{}", line);
        }
    });

    // Wait for the command to finish
    child.wait().await?;

    // Wait for output streaming to complete
    stdout_handle.await?;
    stderr_handle.await?;

    Ok(())
}
