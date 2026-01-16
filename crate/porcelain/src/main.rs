use anyhow::Result;
use gixkit::{open_repo, StatusIterBuilder, UntrackedIterBuilder};
use std::sync::Arc;

fn main() -> Result<()> {
    let repo = open_repo(".")?;
    let repo = Arc::new(repo);

    let status_iter = StatusIterBuilder::new(Arc::clone(&repo))
        .show_untracked(false)
        .build()?;

    for result in status_iter {
        let status = result?;
        println!(
            "{}{} {}",
            char::from(status.index_status),
            char::from(status.worktree_status),
            status.path
        );
    }

    let untracked_iter = UntrackedIterBuilder::new(Arc::clone(&repo))
        .filter(gixkit::UntrackedFilter::Normal)
        .build()?;

    for result in untracked_iter {
        let status = result?;
        println!("?? {}", status.path);
    }

    Ok(())
}
