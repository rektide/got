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
    let work_dir = repo.work_dir().unwrap();
    let mut index = repo.index().unwrap();
    index.write().unwrap();
    for entry in index.entries() {
        let path = entry.path(work_dir);
        let rel_path = entry.path_in_index(gix::index::entry::Stage::Unmodified);
        let path_str = rel_path.to_string();
        let mut index_status = ' ';
        let mut worktree_status = ' ';
        let head_entry = head_tree.lookup_entry(&rel_path);
        let head_id = head_entry.and_then(|e| e.object().ok()).map(|o| o.id());
        if let Some(head_id) = head_id {
            if head_id != *entry.id() {
                index_status = 'M';
            }
        } else {
            index_status = 'A';
        }
        if path.exists() {
            let worktree_content = std::fs::read(path).unwrap();
            let worktree_blob = gix::objs::Blob::from(worktree_content);
            let worktree_id = worktree_blob.id();
            if worktree_id != *entry.id() {
                worktree_status = 'M';
            }
        } else {
            worktree_status = 'D';
        }
        if index_status != ' ' || worktree_status != ' ' {
            println!("{}{} {}", index_status, worktree_status, path_str);
        }
    }
    let ignore = repo.objects().ignore();
    let worktree_path = repo.work_dir().unwrap();
    for entry in ignore.iter(worktree_path, None).unwrap() {
        if entry.status == gix::ignore::Kind::Excluded {
            let path = entry.path.strip_prefix(worktree_path).unwrap();
            println!("?? {}", path.display());
        }
    }
}
