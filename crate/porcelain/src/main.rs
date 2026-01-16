use anyhow::Result;
use gixkit::{open_repo, StatusIterBuilder};

fn main() -> Result<()> {
    let repo = open_repo(".")?;

    let status_iter = StatusIterBuilder::new(&repo)
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

    Ok(())
}
