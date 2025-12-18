use crate::config::EnvConfig;
use std::env;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn run_command(config: &EnvConfig, cmd: &str, args: &[String]) {
    let mut command = Command::new(cmd);
    command.args(args);

    if let Some(ref node_path) = config.node_path {
        inject_path(&mut command, node_path);
        command.env("NODE_PATH", Path::new(node_path).join("node_modules"));
    }

    if let Some(ref mingw_path) = config.mingw_path {
        let bin = Path::new(mingw_path).join("bin");
        inject_path(&mut command, &bin.to_string_lossy());
        command.env("CC", bin.join("gcc.exe"));
        command.env("CXX", bin.join("g++.exe"));
    }

    command
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdin(Stdio::inherit());

    let _ = command.status();
}

fn inject_path(cmd: &mut Command, new_path: &str) {
    if let Ok(current_path) = env::var("PATH") {
        let new = format!("{};{}", new_path, current_path);
        cmd.env("PATH", new);
    } else {
        cmd.env("PATH", new_path);
    }
}
