fn main() {
    let repo = match gix::open(".") {
        Ok(repo) => repo,
        Err(e) => {
            eprintln!("fatal: not a git repository: {}", e);
            std::process::exit(1);
        }
    };

    let head_commit = match repo.head_commit() {
        Ok(commit) => commit,
        Err(e) => {
            eprintln!("fatal: cannot get HEAD commit: {}", e);
            std::process::exit(1);
        }
    };

    let head_tree = head_commit.tree().unwrap();

    let index = repo.index().unwrap();
    let work_dir = repo.work_dir().unwrap();

    for (path, entry) in index.entries_with_paths_by_filter_map(|p, e| Some((p, e))) {
        let path_str = path.to_string();
        let full_path = work_dir.join(&*path_str);

        let mut index_status = ' ';
        let mut worktree_status = ' ';

        let entry_oid = entry.id();

        let path_iter = path.split(|&b| b == b'/');

        let mut buf = Vec::new();
        if let Some(head_entry) = head_tree.lookup_entry(path_iter, &mut buf).unwrap() {
            if let Ok(head_obj) = head_entry.object() {
                let head_oid = head_obj.id();
                if head_oid != entry_oid {
                    index_status = 'M';
                }
            }
        } else {
            index_status = 'A';
        }

        if full_path.exists() {
            if let Ok(content) = std::fs::read(&full_path) {
                let oid = gix_object::compute_hash(
                    gix_hash::Kind::Sha1,
                    gix_object::Kind::Blob,
                    &content,
                );
                if oid != entry_oid {
                    worktree_status = 'M';
                }
            }
        } else {
            worktree_status = 'D';
        }

        if index_status != ' ' || worktree_status != ' ' {
            println!("{}{} {}", index_status, worktree_status, path_str);
        }
    }

    let paths = std::fs::read_dir(work_dir).unwrap();

    for entry in paths {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_file() {
            let rel_path = path.strip_prefix(work_dir).unwrap();
            let rel_path_str = rel_path.to_str().unwrap();
            let rel_path_bstr = gix::bstr::BStr::new(rel_path_str);

            let found = index
                .entries_with_paths_by_filter_map(|p, _e| p == rel_path_bstr)
                .next()
                .is_some();

            if !found {
                println!("?? {}", rel_path.display());
            }
        }
    }
}
