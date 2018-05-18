use std::process::Command;

pub fn new_command() -> Command {
    let mut command = Command::new("git");
    command.arg("--no-pager");
    command
}

pub fn new_command_with_repo_path(repo_path: &str) -> Command {
    let mut command = new_command();
    command.current_dir(repo_path);
    command
}
