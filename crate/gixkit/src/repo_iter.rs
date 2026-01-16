use anyhow::Result;
use gix::{
    bstr::{BStr, BString},
    Repository,
};
use gix_hash::ObjectId;
use gix_object::Kind;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::types::{FileMetadata, FileStatus, StatusChar};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IterMode {
    Tracked,
    Untracked,
    Both,
}

pub struct RepoIterBuilder {
    repo: Arc<Repository>,
    mode: IterMode,
    status_filter: Option<Vec<StatusChar>>,
    include_metadata: bool,
    subdir: Option<PathBuf>,
}

impl RepoIterBuilder {
    pub fn new(repo: Arc<Repository>) -> Self {
        Self {
            repo,
            mode: IterMode::Both,
            status_filter: None,
            include_metadata: false,
            subdir: None,
        }
    }

    pub fn mode(mut self, mode: IterMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn filter(mut self, chars: Vec<StatusChar>) -> Self {
        self.status_filter = Some(chars);
        self
    }

    pub fn include_metadata(mut self, include: bool) -> Self {
        self.include_metadata = include;
        self
    }

    pub fn subdir(mut self, path: impl AsRef<Path>) -> Self {
        self.subdir = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn build(self) -> Result<RepoIter> {
        RepoIter::new(
            self.repo,
            self.mode,
            self.status_filter,
            self.include_metadata,
            self.subdir,
        )
    }
}

pub struct RepoIter {
    repo: Arc<Repository>,
    work_dir: PathBuf,
    head_tree_id: ObjectId,
    tracked_iter: std::vec::IntoIter<(BString, ObjectId)>,
    untracked_dir_stack: Vec<PathBuf>,
    untracked_current_iter: Option<std::fs::ReadDir>,
    mode: IterMode,
    status_filter: Option<Vec<StatusChar>>,
    include_metadata: bool,
    phase: IterationPhase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IterationPhase {
    Tracked,
    Untracked,
}

impl RepoIter {
    fn new(
        repo: Arc<Repository>,
        mode: IterMode,
        status_filter: Option<Vec<StatusChar>>,
        include_metadata: bool,
        subdir: Option<PathBuf>,
    ) -> Result<Self> {
        let work_dir = repo
            .work_dir()
            .ok_or_else(|| anyhow::anyhow!("Repository has no working directory"))?
            .to_path_buf();

        let head_tree_id: ObjectId = {
            let head_tree = crate::get_head_tree(&repo)?;
            head_tree.id().into()
        };

        let tracked_iter = if mode != IterMode::Untracked {
            let index = repo.index()?;
            let entries: Vec<(BString, ObjectId)> = if let Some(ref subdir) = subdir {
                let subdir_str = subdir.to_string_lossy().to_string();
                index
                    .entries()
                    .iter()
                    .filter(|entry| entry.path(&index).to_string().starts_with(&subdir_str))
                    .map(|entry| (entry.path(&index).to_owned(), entry.id))
                    .collect()
            } else {
                index
                    .entries()
                    .iter()
                    .map(|entry| (entry.path(&index).to_owned(), entry.id))
                    .collect()
            };
            entries.into_iter()
        } else {
            Vec::new().into_iter()
        };

        let untracked_start_dir = if let Some(ref subdir) = subdir {
            work_dir.join(subdir)
        } else {
            work_dir.clone()
        };

        let untracked_dir_stack = if mode != IterMode::Tracked {
            vec![untracked_start_dir]
        } else {
            vec![]
        };

        Ok(Self {
            repo,
            work_dir,
            head_tree_id,
            tracked_iter,
            untracked_dir_stack,
            untracked_current_iter: None,
            mode,
            status_filter,
            include_metadata,
            phase: IterationPhase::Tracked,
        })
    }

    pub fn builder(repo: Arc<Repository>) -> RepoIterBuilder {
        RepoIterBuilder::new(repo)
    }

    fn next_tracked(&mut self) -> Option<Result<FileStatus>> {
        while let Some((path, oid)) = self.tracked_iter.next() {
            let file_status = self.compute_file_status(path, oid);

            if file_status.has_changes() && !self.should_filter_out_tracked(&file_status) {
                return Some(Ok(file_status));
            }
        }
        None
    }

    fn should_filter_out_tracked(&self, status: &FileStatus) -> bool {
        if let Some(ref filter) = self.status_filter {
            !filter.contains(&status.index_status) && !filter.contains(&status.worktree_status)
        } else {
            false
        }
    }

    fn compute_file_status(&self, path: BString, entry_oid: ObjectId) -> FileStatus {
        let mut index_status = ' ';
        let mut worktree_status = ' ';

        let head_tree = self.repo.find_tree(self.head_tree_id).ok();

        let path_iter = path.split(|&b| b == b'/');
        let mut buf = Vec::new();

        if let Some(head_tree) = head_tree {
            if let Some(head_entry) = head_tree.lookup_entry(path_iter, &mut buf).ok().flatten() {
                if let Ok(head_obj) = head_entry.object() {
                    let head_oid = head_obj.id();
                    if head_oid != entry_oid {
                        index_status = 'M';
                    }
                }
            } else {
                index_status = 'A';
            }
        } else {
            index_status = 'A';
        }

        let path_str = path.to_string();
        let full_path = self.work_dir.join(&path_str);

        let metadata = if full_path.exists() {
            if let Ok(content) = std::fs::read(&full_path) {
                let file_oid =
                    gix_object::compute_hash(gix::hash::Kind::Sha1, Kind::Blob, &content);
                if file_oid != entry_oid {
                    worktree_status = 'M';
                }
            }
            if self.include_metadata {
                match std::fs::metadata(&full_path) {
                    Ok(m) => Some(FileMetadata {
                        modified_time: m.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH),
                        size: m.len(),
                    }),
                    Err(_) => None,
                }
            } else {
                None
            }
        } else {
            worktree_status = 'D';
            if self.include_metadata {
                Some(FileMetadata {
                    modified_time: std::time::SystemTime::UNIX_EPOCH,
                    size: 0,
                })
            } else {
                None
            }
        };

        FileStatus {
            path: path_str,
            index_status: crate::types::StatusChar::from_char(index_status),
            worktree_status: crate::types::StatusChar::from_char(worktree_status),
            metadata,
        }
    }

    fn next_dir_entry(&mut self) -> Option<Result<std::fs::DirEntry>> {
        loop {
            if self.untracked_current_iter.is_none() {
                if let Some(dir) = self.untracked_dir_stack.pop() {
                    match std::fs::read_dir(&dir) {
                        Ok(iter) => self.untracked_current_iter = Some(iter),
                        Err(e) => return Some(Err(e.into())),
                    }
                } else {
                    return None;
                }
            }

            if let Some(ref mut iter) = self.untracked_current_iter {
                match iter.next() {
                    Some(Ok(entry)) => return Some(Ok(entry)),
                    Some(Err(_)) => continue,
                    None => {
                        self.untracked_current_iter = None;
                        continue;
                    }
                }
            }
        }
    }

    fn next_untracked(&mut self) -> Option<Result<FileStatus>> {
        let index = match self.repo.index() {
            Ok(idx) => idx,
            Err(_) => return Some(Err(anyhow::anyhow!("Failed to get index"))),
        };

        loop {
            let entry_result = self.next_dir_entry();
            let entry = match entry_result {
                Some(Ok(e)) => e,
                Some(Err(e)) => return Some(Err(e)),
                None => return None,
            };

            let path = entry.path();

            let file_name = match path.file_name() {
                Some(name) => name,
                None => continue,
            };

            let file_name_str = match file_name.to_str() {
                Some(s) => s,
                None => continue,
            };

            if file_name_str.starts_with('.') || file_name_str == ".git" {
                continue;
            }

            let rel_path = match path.strip_prefix(&self.work_dir) {
                Ok(p) => p,
                Err(_) => continue,
            };

            let rel_path_str = match rel_path.to_str() {
                Some(s) => s,
                None => continue,
            };

            let rel_path_bstr: &BStr = rel_path_str.into();

            if index.entry_by_path(rel_path_bstr).is_some() {
                continue;
            }

            if path.is_dir() {
                self.untracked_dir_stack.push(path);
                continue;
            }

            let metadata = if self.include_metadata {
                match std::fs::metadata(&path) {
                    Ok(m) => Some(FileMetadata {
                        modified_time: m.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH),
                        size: m.len(),
                    }),
                    Err(_) => None,
                }
            } else {
                None
            };

            let file_status = FileStatus {
                path: rel_path_str.to_string(),
                index_status: crate::types::StatusChar::None,
                worktree_status: crate::types::StatusChar::Untracked,
                metadata,
            };

            if !self.should_filter_out_untracked(&file_status) {
                return Some(Ok(file_status));
            }
        }
    }

    fn should_filter_out_untracked(&self, status: &FileStatus) -> bool {
        if let Some(ref filter) = self.status_filter {
            !filter.contains(&status.index_status) && !filter.contains(&status.worktree_status)
        } else {
            false
        }
    }
}

impl Iterator for RepoIter {
    type Item = Result<FileStatus>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.phase {
                IterationPhase::Tracked => {
                    if self.mode == IterMode::Untracked {
                        self.phase = IterationPhase::Untracked;
                        continue;
                    }

                    if let Some(result) = self.next_tracked() {
                        return Some(result);
                    }

                    if self.mode == IterMode::Tracked {
                        return None;
                    }

                    self.phase = IterationPhase::Untracked;
                }
                IterationPhase::Untracked => {
                    if let Some(result) = self.next_untracked() {
                        return Some(result);
                    }
                    return None;
                }
            }
        }
    }
}
