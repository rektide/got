use std::path::Path;
fn main() {
    let repo = match gix::discover(".") {
        Ok(repo) => repo,
        Err(e) => {
            eprintln!("fatal: not a git repository: {}", e);
            std::process::exit(1);
        }
    };
    let head = match repo.head() {
        Ok(head) => head,
        Err(e) => {
            eprintln!("fatal: cannot get HEAD: {}", e);
            std::process::exit(1);
        }
    };
    let head_commit = match head.try_peel_to_commit() {
        Ok(Some(commit)) => commit,
        Ok(None) => {
            eprintln!("fatal: HEAD is not a commit");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("fatal: cannot peel HEAD: {}", e);
            std::process::exit(1);
        }
    };
    let head_tree = head_commit.tree().unwrap();
    let mut index = repo.index().unwrap();
    let worktree = repo.workdir().unwrap();
    let entries = index.entries_mut();
    for entry in entries.iter_mut() {
        let path = entry.path(workdir);
        if !path.exists() {
            continue;
        }
        let head_oid = head_tree.lookup_entry(&entry.path_in_index(gix::index::entry::Stage::Unmodified))
            .and_then(|e| e.object().ok());
        let worktree_oid = gix::ObjectRef::from(entry.id());
        
        let mut staged = ' ';
        let mut worktree = ' ';
        if let Some(head_obj) = head_oid {
            if head_obj.id() != entry.id() {
                staged = 'M';
            }
        }
        if path.exists() {
            let worktree_content = std::fs::read(path).unwrap();
            let worktree_blob = gix::objs::BlobRef::from(&worktree_content);
            let worktree_id = worktree_blob.id();
            if worktree_id != entry.id() {
                worktree = 'M';
            }
        }
        if staged != ' ' || worktree != ' ' {
            println!("{}{} {}", staged, worktree, entry.path(workdir).display());
        }
    }
    let status = gix::status::Plumbing::new(repo).status();
    for entry in status.worktree.entries() {
        if let gix::status::Entry::Untracked(entry) = entry {
            println!("?? {}", entry.path().display());
        }
    }
}
