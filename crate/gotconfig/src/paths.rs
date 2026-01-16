use directories::BaseDirs;
use std::path::PathBuf;

pub fn xdg_git_dir() -> PathBuf {
    BaseDirs::new()
        .expect("Cannot determine home directory")
        .config_dir()
        .join("git")
}

pub fn xdg_git_config() -> PathBuf {
    xdg_git_dir().join("config")
}

pub fn xdg_git_config_d() -> PathBuf {
    xdg_git_dir().join("config.d")
}
