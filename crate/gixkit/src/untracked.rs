use anyhow::Result;
use gix::{bstr::BStr, Repository};
use std::path::PathBuf;
use std::sync::Arc;

use crate::types::{FileStatus, StatusChar};

/// Builder for UntrackedIter
pub struct UntrackedIterBuilder {
    repo: Arc<Repository>,
    filter: crate::types::UntrackedFilter,
    path_filter: Option<String>,
}

impl UntrackedIterBuilder {
    pub fn new(repo: Arc<Repository>) -> Self {
        Self {
            repo,
            filter: crate::types::UntrackedFilter::Normal,
            path_filter: None,
        }
    }

    pub fn filter(mut self, filter: crate::types::UntrackedFilter) -> Self {
        self.filter = filter;
        self
    }

    pub fn path_filter(mut self, pattern: impl Into<String>) -> Self {
        self.path_filter = Some(pattern.into());
        self
    }

    pub fn build(self) -> Result<UntrackedIter> {
        let repo = self.repo;
        let filter = self.filter;
        UntrackedIter::new(repo, filter)
    }
}

/// Iterator over untracked files
pub struct UntrackedIter {
    repo: Arc<Repository>,
    work_dir: PathBuf,
    dir_stack: Vec<PathBuf>,
    current_dir_iter: Option<std::fs::ReadDir>,
    filter: crate::types::UntrackedFilter,
}

impl UntrackedIter {
    fn new(repo: Arc<Repository>, filter: crate::types::UntrackedFilter) -> Result<Self> {
        let work_dir = repo
            .work_dir()
            .ok_or_else(|| anyhow::anyhow!("Repository has no working directory"))?
            .to_path_buf();

        let dir_stack = if filter != crate::types::UntrackedFilter::No {
            vec![work_dir.clone()]
        } else {
            vec![]
        };

        Ok(Self {
            repo,
            work_dir,
            dir_stack,
            current_dir_iter: None,
            filter,
        })
    }

    pub fn builder(repo: Arc<Repository>) -> UntrackedIterBuilder {
        UntrackedIterBuilder::new(repo)
    }

    fn path_is_tracked(&self, path: &BStr) -> bool {
        let index = match self.repo.index() {
            Ok(idx) => idx,
            Err(_) => return false,
        };

        for entry in index.entries() {
            let entry_path = entry.path(&index);
            if entry_path == path {
                return true;
            }
        }
        false
    }
}

impl Iterator for UntrackedIter {
    type Item = Result<FileStatus>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.current_dir_iter.is_none() {
                if let Some(dir) = self.dir_stack.pop() {
                    match std::fs::read_dir(&dir) {
                        Ok(iter) => self.current_dir_iter = Some(iter),
                        Err(e) => return Some(Err(e.into())),
                    }
                } else {
                    return None;
                }
            }

            if let Some(ref mut iter) = self.current_dir_iter {
                if let Some(entry_result) = iter.next() {
                    let entry = match entry_result {
                        Ok(e) => e,
                        Err(_) => continue,
                    };
                    let path = entry.path();

                    let file_name = path.file_name()?.to_str()?;
                    if file_name.starts_with('.') || file_name == ".git" {
                        continue;
                    }

                    let rel_path = path.strip_prefix(&self.work_dir).ok()?;
                    let rel_path_str = rel_path.to_str()?;
                    let rel_path_bstr = BStr::new(rel_path_str);

                    if self.path_is_tracked(rel_path_bstr) {
                        continue;
                    }

                    if path.is_dir() {
                        if self.filter == crate::types::UntrackedFilter::All {
                            self.dir_stack.push(path);
                        }
                        continue;
                    }

                    return Some(Ok(FileStatus {
                        path: rel_path_str.to_string(),
                        index_status: StatusChar::None,
                        worktree_status: StatusChar::Untracked,
                    }));
                } else {
                    self.current_dir_iter = None;
                }
            }
        }
    }
}
