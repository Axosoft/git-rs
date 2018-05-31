#[cfg(target_os = "macos")]
pub static ENV_PATH_SEPARATOR: &str = ":";
#[cfg(target_os = "linux")]
pub static ENV_PATH_SEPARATOR: &str = ":";
#[cfg(target_os = "windows")]
pub static ENV_PATH_SEPARATOR: &str = ";";
