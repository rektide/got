use anyhow::{Context, Result};
use gotconfig::xdg_git_nah;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn get_nah_path(global: bool, repo_path: Option<&Path>) -> Result<PathBuf> {
    if global {
        Ok(xdg_git_nah())
    } else {
        let repo = repo_path
            .map(Path::to_path_buf)
            .or_else(|| std::env::current_dir().ok())
            .context("Cannot determine repository path")?;
        Ok(repo.join(".git").join("nah"))
    }
}

pub fn read_patterns(global: bool, repo_path: Option<&Path>) -> Result<Vec<String>> {
    let path = get_nah_path(global, repo_path)?;

    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read nah file: {}", path.display()))?;

    Ok(content
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .map(String::from)
        .collect())
}

pub fn add_pattern(pattern: &str, global: bool, repo_path: Option<&Path>) -> Result<()> {
    let path = get_nah_path(global, repo_path)?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("Failed to open nah file: {}", path.display()))?;

    writeln!(file, "{}", pattern)
        .with_context(|| format!("Failed to write pattern to nah file: {}", path.display()))?;

    Ok(())
}

pub fn remove_pattern(pattern: &str, global: bool, repo_path: Option<&Path>) -> Result<()> {
    let path = get_nah_path(global, repo_path)?;

    if !path.exists() {
        return Ok(());
    }

    let patterns = read_patterns(global, repo_path)?;
    let filtered: Vec<_> = patterns.into_iter().filter(|p| p != pattern).collect();

    if filtered.is_empty() {
        fs::remove_file(&path)
            .with_context(|| format!("Failed to remove nah file: {}", path.display()))?;
    } else {
        let content = filtered.join("\n");
        fs::write(&path, content)
            .with_context(|| format!("Failed to write nah file: {}", path.display()))?;
    }

    Ok(())
}

pub fn list_patterns(global: bool, repo_path: Option<&Path>) -> Result<Vec<String>> {
    read_patterns(global, repo_path)
}
