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

pub fn verify_string_is_sha(maybe_sha: &str) -> bool {
    if !maybe_sha.is_ascii() {
        return false;
    }

    if maybe_sha.len() != 40 {
        return false;
    }

    maybe_sha
        .to_lowercase()
        .chars()
        .fold(true, |is_sha, next_char| is_sha && next_char.is_digit(16))
}
