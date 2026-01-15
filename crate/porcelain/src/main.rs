fn main() {
    let repo = match gix::discover(".") {
        Ok(repo) => repo,
        Err(e) => {
            eprintln!("fatal: not a git repository: {}", e);
            std::process::exit(1);
        }
    };

    let head = match repo.head().into_fully_peeled_id() {
        Ok(Some(id)) => id,
        Ok(None) => {
            eprintln!("fatal: HEAD is not a commit");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("fatal: cannot get HEAD: {}", e);
            std::process::exit(1);
        }
    };

    let head_commit = match repo.find_object(head) {
        Ok(commit) => commit,
        Err(e) => {
            eprintln!("fatal: cannot find HEAD commit: {}", e);
            std::process::exit(1);
        }
    };

    let head_tree = match head_commit.peel_to_tree() {
        Ok(tree) => tree,
        Err(e) => {
            eprintln!("fatal: cannot peel to tree: {}", e);
            std::process::exit(1);
        }
    };

    let index = repo.index();
    let entries = index.entries();

    for entry in entries {
        let path = entry.path(gix::index::entry::Mode::FILE);
        let path_str = path.to_string();

        let mut index_status = ' ';
        let mut worktree_status = ' ';

        let head_entry = head_tree.lookup_entry(&path);

        let entry_oid = entry.id;

        if let Some(head_entry) = head_entry {
            if let Ok(head_obj) = head_entry.object() {
                let head_oid = head_obj.id();
                if head_oid != entry_oid {
                    index_status = 'M';
                }
            }
        } else {
            index_status = 'A';
        }

        let work_dir = repo.work_dir().unwrap();
        let full_path = work_dir.join(&path_str);

        if full_path.exists() {
            let worktree_content = std::fs::read(&full_path).unwrap();
            let worktree_oid = gix::hash::ObjectId::hash(gix::hash::Kind::Sha1, &worktree_content);

            if worktree_oid != entry_oid {
                worktree_status = 'M';
            }
        } else {
            worktree_status = 'D';
        }

        if index_status != ' ' || worktree_status != ' ' {
            println!("{}{} {}", index_status, worktree_status, path_str);
        }
    }

    let work_dir = repo.work_dir().unwrap();
    let mut options = gix::status::Options::default();
    let mut status = gix::status::Kind::WorktreeThenIndex;

    let status_output = gix::status::Plumbing::new(&repo).status();

    for entry in status_output.worktree.entries() {
        match entry {
            gix::status::Entry::Untracked(entry) => {
                println!("?? {}", entry.path().display());
            }
            _ => {}
        }
    }
}
