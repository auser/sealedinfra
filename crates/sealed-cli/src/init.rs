use sealed_common::{
    error::SealedResult,
    settings::{get_config, Settings, CONFIG_INSTANCE},
};

use crate::Cli;

pub fn init_config(cli: &Cli) -> SealedResult<&'static Settings> {
    let settings = match &cli.settings {
        None => Settings::from_root(cli.root.clone())?,
        Some(settings) => Settings::from_root(Some(settings.clone()))?,
    };
    CONFIG_INSTANCE
        .set(settings)
        .expect("Config already initialized");
    get_config()
}

impl From<Cli> for Settings {
    fn from(args: Cli) -> Self {
        let root = match args.root {
            None => std::env::current_dir().expect("Unable to get current directory"),
            Some(root) => root,
        };
        Settings::from_root(Some(root)).expect("Unable to get settings")
    }
}
