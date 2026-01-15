use std::path::Path;

fn main() {
    let repo = match gix::discover(".") {
        Ok(repo) => repo,
        Err(e) => {
            eprintln!("fatal: not a git repository: {}", e);
            std::process::exit(1);
        }
    };

    let status = gix::status::Plumbing::new(repo);
    let output = status.status();

    for entry in output.worktree.entries() {
        match entry {
            gix::status::Entry::Untracked(entry) => {
                println!("?? {}", entry.path().display());
            }
            gix::status::Entry::Conflict(_) => {
                // Handle conflicts
            }
            gix::status::Entry::Tracked(entry) => {
                let index_status = if entry.is_index_new() {
                    'A'
                } else if entry.is_index_deleted() {
                    'D'
                } else if entry.is_index_modified() {
                    'M'
                } else {
                    ' '
                };

                let worktree_status = if entry.is_worktree_new() {
                    'A'
                } else if entry.is_worktree_deleted() {
                    'D'
                } else if entry.is_worktree_modified() {
                    'M'
                } else {
                    ' '
                };

                if index_status != ' ' || worktree_status != ' ' {
                    println!("{}{} {}", index_status, worktree_status, entry.path().display());
                }
            }
        }
    }
}
