use anyhow::Result;
use gix::{bstr::BStr, Repository};
use gix_hash::ObjectId;

/// Index entry with path and OID
#[derive(Debug, Clone)]
pub struct IndexedFile {
    pub path: String,
    pub oid: ObjectId,
}

/// Get all entries from index
pub fn get_index_entries(repo: &Repository) -> Result<Vec<IndexedFile>> {
    let index = repo.index()?;

    let mut entries = Vec::new();
    let state = index.state();
    for entry in index.entries() {
        entries.push(IndexedFile {
            path: entry.path(state, std::path::MAIN_SEPARATOR).to_string(),
            oid: entry.id,
        });
    }

    Ok(entries)
}

/// Check if a path exists in index
pub fn path_in_index(repo: &Repository, path: &BStr) -> Result<bool> {
    let index = repo.index()?;

    let mut found = false;
    for (p, _) in index.entries_with_paths_by_filter_map(|p, e| Some((p, e))) {
        if p == path {
            found = true;
            break;
        }
    }

    Ok(found)
}
