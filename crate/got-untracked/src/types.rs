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

/// Untracked file filter options
#[derive(Debug, Clone, Copy, Default)]
pub enum UntrackedFilter {
    /// Show no untracked files
    No,
    /// Show untracked files (normal)
    #[default]
    Normal,
    /// Show all untracked files (including in subdirectories)
    All,
}

/// Configuration for status operations
#[derive(Debug, Clone, Default)]
pub struct StatusOptions {
    pub untracked: UntrackedFilter,
    pub show_ignored: bool,
    pub recurse_submodules: bool,
}

/// Represents status of a file in git
#[derive(Debug, Clone, PartialEq)]
pub struct FileStatus {
    pub path: String,
    pub index_status: StatusChar,
    pub worktree_status: StatusChar,
}
