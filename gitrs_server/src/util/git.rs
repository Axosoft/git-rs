use config;
use constants;
use std::env;
use std::process::Command;

pub fn new_command() -> Command {
    let path = match config::CONFIG.read().unwrap().git_path {
        Some(ref git_path) => {
            let mut git_path = String::from(git_path.clone());
            git_path.push_str(constants::platform::ENV_PATH_SEPARATOR);
            match env::var("PATH") {
                Ok(env_path) => {
                    git_path.push_str(&env_path);
                    Some(git_path)
                }
                Err(_) => None,
            }
        }
        None => None,
    };

    let exec_path = match config::CONFIG.read().unwrap().exec_path {
        Some(ref git_exec_path) => {
            Some(String::from(git_exec_path.clone()))
        }
        None => None,
    };

    let mut command = Command::new("git");
    path.map(|path| {
        println!("{}", &path);
        command.env("PATH", &String::from(path));
    });
    exec_path.map(|exec_path| {
        println!("{}", &exec_path);
        command.env("GIT_EXEC_PATH", &String::from(exec_path));
    });
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
