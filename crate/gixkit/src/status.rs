use anyhow::Result;
use gix::{bstr::BStr, Repository};
use gix_object::Kind;
use std::path::PathBuf;

use crate::types::{FileStatus, StatusChar};

type BString = gix::bstr::BString;

/// Builder for StatusIter
pub struct StatusIterBuilder<'repo> {
    repo: &'repo Repository,
    show_untracked: bool,
    path_filter: Option<String>,
}

impl<'repo> StatusIterBuilder<'repo> {
    pub fn new(repo: &'repo Repository) -> Self {
        Self {
            repo,
            show_untracked: true,
            path_filter: None,
        }
    }

    pub fn show_untracked(mut self, show: bool) -> Self {
        self.show_untracked = show;
        self
    }

    pub fn path_filter(mut self, pattern: impl Into<String>) -> Self {
        self.path_filter = Some(pattern.into());
        self
    }

    pub fn build(self) -> Result<StatusIter<'repo>> {
        StatusIter::new(self.repo, self)
    }
}

/// Iterator over file statuses in repository
pub struct StatusIter<'repo> {
    repo: &'repo Repository,
    show_untracked: bool,
    head_tree: gix::Tree<'repo>,
    work_dir: PathBuf,
    index_iter: Box<dyn Iterator<Item = (BString, gix_hash::ObjectId)> + 'repo>,
    untracked_started: bool,
}

impl<'repo> StatusIter<'repo> {
    fn new(repo: &'repo Repository, builder: StatusIterBuilder<'repo>) -> Result<Self> {
        let head_tree = crate::get_head_tree(repo)?;
        let work_dir = repo
            .work_dir()
            .ok_or_else(|| anyhow::anyhow!("Repository has no working directory"))?
            .to_path_buf();

        let index = repo.index()?;
        let index_iter =
            Box::new(index.entries_with_paths_by_filter_map(|p, e| Some((p.to_owned(), e.id))));

        Ok(Self {
            repo,
            show_untracked: builder.show_untracked,
            head_tree,
            work_dir,
            index_iter,
            untracked_started: false,
        })
    }

    pub fn builder(repo: &'repo Repository) -> StatusIterBuilder<'repo> {
        StatusIterBuilder::new(repo)
    }

    fn compute_index_status(&self, path: BString, entry_oid: gix_hash::ObjectId) -> (char, char) {
        let mut index_status = ' ';
        let mut worktree_status = ' ';

        let path_slice = path.as_ref();
        let path_iter = path_slice.split(|&b| b == b'/');
        let mut buf = Vec::new();

        if let Some(head_entry) = self
            .head_tree
            .lookup_entry(path_iter, &mut buf)
            .ok()
            .flatten()
        {
            if let Ok(head_obj) = head_entry.object() {
                let head_oid = head_obj.id();
                if head_oid != entry_oid {
                    index_status = 'M';
                }
            }
        } else {
            index_status = 'A';
        }

        let path_str = path.to_string();
        let full_path = self.work_dir.join(&path_str);

        if full_path.exists() {
            if let Ok(content) = std::fs::read(&full_path) {
                let oid = gix_object::compute_hash(gix::hash::Kind::Sha1, Kind::Blob, &content);
                if oid != entry_oid {
                    worktree_status = 'M';
                }
            }
        } else {
            worktree_status = 'D';
        }

        (index_status, worktree_status)
    }
}

impl<'repo> Iterator for StatusIter<'repo> {
    type Item = Result<FileStatus>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((path, entry_oid)) = self.index_iter.next() {
            let (index_status, worktree_status) =
                self.compute_index_status(path.clone(), entry_oid);

            let file_status = FileStatus {
                path: path.to_string(),
                index_status: StatusChar::from_char(index_status),
                worktree_status: StatusChar::from_char(worktree_status),
            };

            if file_status.has_changes() {
                return Some(Ok(file_status));
            }
        }

        if self.show_untracked && !self.untracked_started {
            self.untracked_started = true;
        }

        None
    }
}
