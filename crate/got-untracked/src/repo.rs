use anyhow::Result;
use gix::Repository;

/// Open a repository with proper error handling
pub fn open_repo(path: &str) -> Result<Repository> {
    gix::open(path).map_err(|e| anyhow::anyhow!("Failed to open git repository '{}': {}", path, e))
}

/// Get HEAD commit, falling back to empty tree for empty repos
pub fn get_head_tree(repo: &Repository) -> Result<gix::Tree> {
    match repo.head_commit() {
        Ok(commit) => commit
            .tree()
            .map_err(|e| anyhow::anyhow!("Failed to get HEAD tree: {}", e)),
        Err(_) => {
            let oid = gix_hash::ObjectId::empty_tree(gix::hash::Kind::Sha1);
            repo.find_tree(oid)
                .map_err(|e| anyhow::anyhow!("Failed to find empty tree: {}", e))
        }
    }
}
