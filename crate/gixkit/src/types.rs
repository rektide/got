#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusChar {
    Modified,
    Added,
    Deleted,
    Renamed,
    Copied,
    Unmerged,
    Untracked,
    Ignored,
    None,
}

impl From<StatusChar> for char {
    fn from(c: StatusChar) -> char {
        match c {
            StatusChar::Modified => 'M',
            StatusChar::Added => 'A',
            StatusChar::Deleted => 'D',
            StatusChar::Renamed => 'R',
            StatusChar::Copied => 'C',
            StatusChar::Unmerged => 'U',
            StatusChar::Untracked => '?',
            StatusChar::Ignored => '!',
            StatusChar::None => ' ',
        }
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

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum UntrackedFilter {
    No,
    #[default]
    Normal,
    All,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileMetadata {
    pub modified_time: std::time::SystemTime,
    pub size: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileStatus {
    pub path: String,
    pub index_status: StatusChar,
    pub worktree_status: StatusChar,
    pub metadata: Option<FileMetadata>,
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
