use crate::cli::GoldestArgs;
use anyhow::Result;
use gixkit::{open_repo, IterMode, RepoIterBuilder};
use std::sync::Arc;

pub fn execute(args: GoldestArgs) -> Result<()> {
    let repo = open_repo(std::env::current_dir()?)?;
    let repo = Arc::new(repo);

    let show_untracked = args.untracked.is_some();
    let mode = if show_untracked {
        IterMode::Both
    } else {
        IterMode::Tracked
    };

    let repo_iter = RepoIterBuilder::new(Arc::clone(&repo))
        .mode(mode)
        .include_metadata(true)
        .build()?;

    let mut files: Vec<_> = repo_iter.collect::<Result<Vec<_>>>()?;

    files.sort_by_key(|f| {
        f.metadata
            .as_ref()
            .map(|m| m.modified_time)
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });

    let skip = args.skip;
    let lines = args.lines;
    let files = files.into_iter().skip(skip).take(lines).collect::<Vec<_>>();

    for file in files {
        let metadata = file
            .metadata
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing metadata"))?;
        let modified_time = chrono::DateTime::<chrono::Utc>::from(metadata.modified_time)
            .format("%m-%d-%yT%H:%M:%SZ")
            .to_string();

        if args.file_only {
            println!("{}", file.path);
        } else if args.date_only {
            println!("{}", modified_time);
        } else if args.short {
            let index_char: char = file.index_status.into();
            let worktree_char: char = file.worktree_status.into();
            println!(
                "{}{} {} {}",
                index_char, worktree_char, modified_time, file.path
            );
        } else if args.porcelain {
            let index_char: char = file.index_status.into();
            let worktree_char: char = file.worktree_status.into();
            println!(
                "{}{} {} {} {}",
                index_char, worktree_char, modified_time, file.path, metadata.size
            );
        } else {
            println!("{} {}", file.path, modified_time);
        }
    }

    Ok(())
}
