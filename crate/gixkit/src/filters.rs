use crate::types::{FileStatus, StatusChar};

/// Iterator that filters by status
pub struct StatusFilterIter<I> {
    inner: I,
    filter: StatusFilter,
}

#[derive(Debug, Clone, Copy)]
pub enum StatusFilter {
    /// Only files with changes
    Changed,
    /// Only staged files
    Staged,
    /// Only worktree changes
    Worktree,
    /// Files with specific index status
    IndexStatus(StatusChar),
    /// Files with specific worktree status
    WorktreeStatus(StatusChar),
    /// Custom predicate
    Custom(fn(&FileStatus) -> bool),
}

impl<I> StatusFilterIter<I> {
    pub fn new(inner: I, filter: StatusFilter) -> Self {
        Self { inner, filter }
    }
}

impl<I> Iterator for StatusFilterIter<I>
where
    I: Iterator<Item = anyhow::Result<FileStatus>>,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next()? {
                Ok(status) => {
                    let matches = match self.filter {
                        StatusFilter::Changed => status.has_changes(),
                        StatusFilter::Staged => status.is_staged(),
                        StatusFilter::Worktree => status.is_worktree_modified(),
                        StatusFilter::IndexStatus(c) => status.index_status == c,
                        StatusFilter::WorktreeStatus(c) => status.worktree_status == c,
                        StatusFilter::Custom(f) => f(&status),
                    };

                    if matches {
                        return Some(Ok(status));
                    }
                }
                Err(e) => return Some(Err(e)),
            }
        }
    }
}
