use std::path::PathBuf;

fn main() {
    let repo = match gix::discover(".") {
        Ok(repo) => repo,
        Err(e) => {
            eprintln!("fatal: not a git repository: {}", e);
            std::process::exit(1);
        }
    };

    let head = repo.head().unwrap();
    let head_commit = match head.try_peel_to_commit() {
        Ok(Some(commit)) => commit,
        _ => {
            eprintln!("fatal: cannot get HEAD commit");
            std::process::exit(1);
        }
    };

    let head_tree = head_commit.tree().unwrap();
    let index = repo.index().unwrap();
    let work_dir = repo.work_dir().unwrap();

    let mut index_mut = index.clone();
    index_mut.write(gix::index::write::Options::default()).unwrap();

    let entries = index.entries();

    for entry in entries {
        let path = entry.path(gix::index::entry::Mode::FILE);
        let path_str = path.to_string();
        let full_path: PathBuf = work_dir.join(&path_str);

        let mut index_status = ' ';
        let mut worktree_status = ' ';

        let head_entry = head_tree.lookup_entry(&path);

        let entry_oid = entry.id();

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

        if full_path.exists() {
            if let Ok(content) = std::fs::read(&full_path) {
                let oid = gix::hash::ObjectId::sha1(&content);
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

    let git_dir = repo.git_dir();
    let paths = std::fs::read_dir(work_dir).unwrap();

    for entry in paths {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_file() {
            let rel_path = path.strip_prefix(work_dir).unwrap();
            let rel_path_bstr = gix::bstr::BStr::new(rel_path.to_str().unwrap());

            if !index.entries().iter().any(|e| e.path(gix::index::entry::Mode::FILE) == rel_path_bstr) {
                println!("?? {}", rel_path.display());
            }
        }
    }
}
