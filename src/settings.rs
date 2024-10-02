use std::{env, path::PathBuf, sync::OnceLock};

use anyhow::Context;
use config::File;
use log::LevelFilter;
use serde::{Deserialize, Serialize};
use std::fs::canonicalize;

use crate::cmd::Cli;
use crate::error::SealedResult;

pub static CONFIG_INSTANCE: OnceLock<Settings> = OnceLock::new();

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ServerArgs {
    pub port: u16,
}

impl Default for ServerArgs {
    fn default() -> Self {
        Self { port: 9999 }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Settings {
    #[serde(default = "default_log_level")]
    pub log_level: LevelFilter,

    #[serde(default = "default_working_directory")]
    pub working_directory: PathBuf,

    #[serde(default = "default_ssh_key")]
    pub ssh_key: Option<PathBuf>,

    #[serde(default = "ServerArgs::default")]
    pub server: ServerArgs,
}

pub fn get_config() -> SealedResult<&'static Settings> {
    Ok(CONFIG_INSTANCE.get().expect("Config not initialized"))
}

pub fn init_config(cli: &Cli) -> SealedResult<&'static Settings> {
    let root = match &cli.settings {
        None => PathBuf::from(&cli.root.clone().unwrap()),
        Some(settings) => settings.clone(),
    };
    let settings = Settings::from_root(Some(root))?;
    CONFIG_INSTANCE
        .set(settings)
        .expect("Config already initialized");
    get_config()
}

impl Settings {
    pub fn from_root(root: Option<PathBuf>) -> SealedResult<Self> {
        let curr_dir = std::env::current_dir().context("unable to get working directory")?;
        let root = root.unwrap_or(curr_dir);
        let root = canonicalize(root).context("unable to canonicalize root directory")?;

        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".to_string());

        let s = config::Config::builder()
            .add_source(File::from(root.as_path()))
            .add_source(File::with_name("config").required(false))
            .add_source(File::with_name("config/default").required(false))
            .add_source(File::with_name(&format!("config.{}", run_mode)).required(false))
            .add_source(
                File::with_name(&format!("{}/config", root.as_path().to_str().unwrap()))
                    .required(false),
            )
            .build()?;

        let cfg = s.try_deserialize()?;
        Ok(cfg)
    }
}

impl From<Cli> for Settings {
    fn from(args: Cli) -> Self {
        Settings::from_root(args.root).expect("Unable to get settings")
    }
}

fn default_log_level() -> LevelFilter {
    LevelFilter::Info
}

fn default_working_directory() -> PathBuf {
    PathBuf::from("/tmp")
}

fn default_ssh_key() -> Option<PathBuf> {
    let home = env::var("HOME").unwrap();
    Some(PathBuf::from(format!("{home}/.ssh/id_rsa")))
}
