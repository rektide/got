use anyhow::Result;
use gixkit::{open_repo, IterMode, RepoIterBuilder};
use std::sync::Arc;

fn main() -> Result<()> {
    let repo = open_repo(".")?;
    let repo = Arc::new(repo);

    let iter = RepoIterBuilder::new(Arc::clone(&repo))
        .mode(IterMode::Both)
        .build()?;

    for result in iter {
        let status = result?;
        println!(
            "{}{} {}",
            char::from(status.index_status),
            char::from(status.worktree_status),
            status.path
        );
    }

    Ok(())
}
