use std::{fmt::Display, path::PathBuf};

use std::process::Command;

use crate::{error::SealedResult, settings::Settings, util::git_ops::parse_repo_name};

use super::DockerHandlerArgs;

pub fn build_docker_build_command(
    args: DockerHandlerArgs,
    repo_root: &PathBuf,
    _settings: &Settings,
) -> SealedResult<Command> {
    let mut command: Command = args.into();
    command.current_dir(repo_root);
    println!("Command: {:?}", command);
    Ok(command)
}

impl From<DockerHandlerArgs> for Command {
    fn from(args: DockerHandlerArgs) -> Self {
        let mut command = Command::new("docker");
        // command.arg("run");
        // if args.rm {
        //     command.arg("--rm");
        // }
        // for bind in args.binds {
        //     let bind = DockerBind::from(bind);
        //     command.arg("-v");
        //     command.arg(format!(
        //         "{}:{}:{}",
        //         bind.host_path, bind.container_path, bind.mode
        //     ));
        // }
        // for volume in args.volumes {
        //     let volume = DockerBind::from(volume);
        //     command.arg("-v");
        //     command.arg(format!(
        //         "{}:{}:{}",
        //         volume.host_path, volume.container_path, volume.mode
        //     ));
        // }
        // for env in args.env {
        //     let env = DockerEnv::from(env);
        //     command.arg("-e");
        //     command.arg(format!("{}={}", env.key, env.value));
        // }
        // if let Some(name) = args.name {
        //     command.arg("--name");
        //     command.arg(name);
        // }
        // if let Some(user) = args.user {
        //     command.arg("--user");
        //     command.arg(user);
        // }
        // let img = match args.docker.image {
        //     Some(image) => image,
        //     None => parse_repo_name(&args.docker.repository.unwrap()).unwrap(),
        // };
        // command.arg(format!(
        //     "-t {}:{}",
        //     img,
        //     args.docker.tag.unwrap_or("latest".to_string())
        // ));

        // command.arg("build");

        command
    }
}
