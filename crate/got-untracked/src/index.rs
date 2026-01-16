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

    index
        .entries_with_paths_by_filter_map(|p, e| {
            Some(IndexedFile {
                path: p.to_string(),
                oid: e.id,
            })
        })
        .collect()
}

/// Check if a path exists in index
pub fn path_in_index(repo: &Repository, path: &BStr) -> Result<bool> {
    let index = repo.index()?;

    Ok(index
        .entries_with_paths_by_filter_map(|p, _e| if p == path { Some(()) } else { None })
        .next()
        .is_some())
}
