use gix_hash::ObjectId;

/// Git status character
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusChar {
    Modified = 'M',
    Added = 'A',
    Deleted = 'D',
    Renamed = 'R',
    Copied = 'C',
    Unmerged = 'U',
    Untracked = '?',
    Ignored = '!',
    None = ' ',
}

impl From<StatusChar> for char {
    fn from(c: StatusChar) -> char {
        c as u8 as char
    }
}

impl StatusChar {
    pub fn from_char(c: char) -> Self {
        match c {
            'M' => StatusChar::Modified,
            'A' => StatusChar::Added,
            'D' => StatusChar::Deleted,
            'R' => StatusChar::Renamed,
            'C' => StatusChar::Copied,
            'U' => StatusChar::Unmerged,
            '?' => StatusChar::Untracked,
            '!' => StatusChar::Ignored,
            ' ' => StatusChar::None,
            _ => StatusChar::None,
        }
    }
}

/// Untracked file filter options
#[derive(Debug, Clone, Copy, Default)]
pub enum UntrackedFilter {
    /// Show no untracked files
    No,
    /// Show untracked files (normal - no recursion)
    #[default]
    Normal,
    /// Show all untracked files (including in subdirectories)
    All,
}

/// Represents status of a file in git
#[derive(Debug, Clone, PartialEq)]
pub struct FileStatus {
    pub path: String,
    pub index_status: StatusChar,
    pub worktree_status: StatusChar,
}

impl FileStatus {
    pub fn has_changes(&self) -> bool {
        self.index_status != StatusChar::None || self.worktree_status != StatusChar::None
    }

    pub fn is_staged(&self) -> bool {
        self.index_status != StatusChar::None
    }

    pub fn is_worktree_modified(&self) -> bool {
        self.worktree_status != StatusChar::None
    }
}

/// Extended file status with additional metadata
#[derive(Debug, Clone)]
pub struct FileStatusExt {
    pub base: FileStatus,
    pub index_oid: Option<ObjectId>,
    pub worktree_oid: Option<ObjectId>,
    pub head_oid: Option<ObjectId>,
    pub worktree_path: std::path::PathBuf,
}

impl FileStatusExt {
    pub fn from_base(base: FileStatus, repo: &gix::Repository) -> anyhow::Result<Self> {
        let work_dir = repo
            .work_dir()
            .ok_or_else(|| anyhow::anyhow!("Repository has no working directory"))?;

        let worktree_path = work_dir.join(&base.path);

        Ok(Self {
            base,
            index_oid: None,
            worktree_oid: None,
            head_oid: None,
            worktree_path,
        })
    }
}
