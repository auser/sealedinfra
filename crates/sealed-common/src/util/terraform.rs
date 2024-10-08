use std::process::Stdio;

use tokio::process::Command;

use crate::error::SealedResult;

#[derive(Debug, Clone, Default)]
pub struct TerraformOptions {
    pub dir: Option<String>,
}

impl TerraformOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_dir<T: Into<String>>(&mut self, dir: Option<T>) -> &mut Self {
        if let Some(dir) = dir {
            self.dir = Some(dir.into());
        }
        self
    }

    pub fn build(self) -> Self {
        let mut opts = TerraformOptions::default();
        if let Some(dir) = self.dir {
            opts.dir = Some(dir);
        }
        opts
    }
}

pub async fn init_terraform(opts: &TerraformOptions) -> SealedResult<()> {
    let mut cmd = TerraformCommandBuilder::new("init")
        .with_dir(opts.dir.clone())
        .build();

    cmd.stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Failed to spawn terraform init command")
        .wait()
        .await?;
    Ok(())
}

#[derive(Debug, Default)]
pub struct TerraformCommandBuilder {
    pub cmd: String,
    pub dir: Option<String>,
}

impl TerraformCommandBuilder {
    pub fn new<T: Into<String>>(cmd: T) -> Self {
        Self {
            cmd: cmd.into(),
            dir: None,
        }
    }

    pub fn with_dir<T: Into<String>>(&mut self, dir: Option<T>) -> &Self {
        if let Some(dir) = dir {
            self.dir = Some(dir.into());
        }
        self
    }

    pub fn build(&self) -> Command {
        let mut cmd = Command::new(self.cmd.clone());
        if let Some(dir) = self.dir.clone() {
            cmd.current_dir(dir);
        }
        cmd
    }
}
