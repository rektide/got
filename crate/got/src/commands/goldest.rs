use crate::cli::GoldestArgs;
use anyhow::Result;
use gixkit::{open_repo, DateIter, IterMode, RepoIterBuilder};
use std::sync::Arc;

pub fn execute(args: GoldestArgs) -> Result<()> {
    let repo = open_repo(std::env::current_dir()?)?;
    let repo = Arc::new(repo);

    let work_dir = repo
        .work_dir()
        .ok_or_else(|| anyhow::anyhow!("Repository has no working directory"))?
        .to_path_buf();

    let show_untracked = args.untracked.is_some();
    let mode = if show_untracked {
        IterMode::Both
    } else {
        IterMode::Tracked
    };

    let repo_iter = RepoIterBuilder::new(Arc::clone(&repo)).mode(mode).build()?;

    let date_iter = DateIter::new(repo_iter, work_dir);

    let mut files: Vec<_> = date_iter.collect::<Result<Vec<_>>>()?;

    files.sort_by_key(|f| f.modified_time);

    let skip = args.skip;
    let lines = args.lines;
    let files = files.into_iter().skip(skip).take(lines).collect::<Vec<_>>();

    for file in files {
        let modified_time = chrono::DateTime::<chrono::Utc>::from(file.modified_time)
            .format("%m-%d-%yT%H:%M:%SZ")
            .to_string();

        if args.file_only {
            println!("{}", file.status.path);
        } else if args.date_only {
            println!("{}", modified_time);
        } else if args.short {
            let index_char: char = file.status.index_status.into();
            let worktree_char: char = file.status.worktree_status.into();
            println!(
                "{}{} {} {}",
                index_char, worktree_char, modified_time, file.status.path
            );
        } else if args.porcelain {
            let index_char: char = file.status.index_status.into();
            let worktree_char: char = file.status.worktree_status.into();
            println!(
                "{}{} {} {} {}",
                index_char, worktree_char, modified_time, file.status.path, file.size
            );
        } else {
            println!("{} {}", file.status.path, modified_time);
        }
    }

    Ok(())
}
