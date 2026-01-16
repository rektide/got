# gixkit

Git iterator library for the `got` toolkit.

## Overview

`gixkit` provides composable, lazy iterators for git operations built on top of the `gix` library. It abstracts common patterns for tracking file status, untracked files, and other git operations into reusable iterator components.

### Key Features

- **Lazy evaluation**: Iterators avoid loading entire repository state into memory
- **Builder pattern**: Flexible configuration of iterator behavior
- **Composable**: Filter and decorate iterators using standard iterator combinators
- **Error-per-item**: Individual iteration failures don't abort entire operation
- **Borrowed from repo**: Lifetime-bound to repository for zero-copy operation

## Motivation

The `got` toolkit has multiple commands that need git status information:

- `got nah pick` - Iterate over untracked files
- `got statusd` - Iterate over file status with modification times
- `got goldest` - Find oldest modified files
- `porcelain` - Display git status in porcelain format

Without `gixkit`, each implementation would duplicate repository handling, index iteration, tree lookups, and status computation. `gixkit` centralizes this logic into a shared, well-tested library.

## Installation

Add to `Cargo.toml`:

```toml
[dependencies]
gixkit = { path = "../gixkit" }
```

## Core Concepts

### Status Character

Represents the status of a file in a particular area (index or worktree):

```rust
use gixkit::StatusChar;

pub enum StatusChar {
    Modified = 'M',  // File has been modified
    Added = 'A',      // File has been added
    Deleted = 'D',     // File has been deleted
    Renamed = 'R',     // File has been renamed
    Copied = 'C',      // File has been copied
    Unmerged = 'U',     // File has unmerged changes
    Untracked = '?',    // File is untracked
    Ignored = '!',      // File is ignored
    None = ' ',         // No changes
}

// Convert to char
let c: char = StatusChar::Modified.into();
```

### File Status

Represents the combined status of a file:

```rust
use gixkit::FileStatus;

pub struct FileStatus {
    pub path: String,
    pub index_status: StatusChar,    // Staged status (XY: X column)
    pub worktree_status: StatusChar, // Worktree status (XY: Y column)
}

impl FileStatus {
    // File has any changes
    pub fn has_changes(&self) -> bool;

    // File has staged changes
    pub fn is_staged(&self) -> bool;

    // File has worktree modifications
    pub fn is_worktree_modified(&self) -> bool;
}
```

### Untracked Filter

Controls which untracked files to include:

```rust
use gixkit::UntrackedFilter;

pub enum UntrackedFilter {
    No,      // Show no untracked files
    Normal,   // Show untracked files (no recursion)
    All,      // Show all untracked files (recursive)
}
```

## API Reference

### Repository Operations

#### Opening Repositories

```rust
use gixkit::open_repo;

let repo = open_repo(".")?;

// With explicit path
let repo = open_repo("/path/to/repo")?;
```

#### Getting HEAD Tree

```rust
use gixkit::get_head_tree;

let head_tree = get_head_tree(&repo)?;
// Returns empty tree for new repositories
```

### Index Operations

#### Get All Index Entries

```rust
use gixkit::get_index_entries;

let entries = get_index_entries(&repo)?;

for entry in entries {
    println!("{}: {}", entry.path, entry.oid);
}
```

#### Check if Path is Tracked

```rust
use gixkit::path_in_index;
use gix::bstr::BStr;

let path = BStr::new("src/main.rs");
if path_in_index(&repo, &path)? {
    println!("File is tracked");
}
```

### Status Iterator

Iterate over tracked files with status information:

```rust
use gixkit::StatusIterBuilder;

let status_iter = StatusIterBuilder::new(&repo)
    .show_untracked(false)
    .build()?;

for result in status_iter {
    let status = result?;
    println!("{}{} {}",
        char::from(status.index_status),
        char::from(status.worktree_status),
        status.path
    );
}
```

#### Builder Options

```rust
StatusIterBuilder::new(&repo)
    .show_untracked(true)          // Include untracked files
    .untracked_filter(UntrackedFilter::All)  // Recursive untracked
    .path_filter("*.rs")           // Filter by glob pattern
    .build()?;
```

### Untracked Iterator

Iterate over untracked files:

```rust
use gixkit::UntrackedIterBuilder;

let untracked_iter = UntrackedIterBuilder::new(&repo)
    .filter(UntrackedFilter::All)  // Include subdirectories
    .path_filter("*.tmp")         // Only .tmp files
    .build()?;

for result in untracked_iter {
    let status = result?;
    println!("?? {}", status.path);
}
```

#### Builder Options

```rust
UntrackedIterBuilder::new(&repo)
    .filter(UntrackedFilter::Normal)   // Top-level only
    .filter(UntrackedFilter::All)      // Recursive
    .filter(UntrackedFilter::No)        // No untracked files
    .path_filter("pattern")            // Glob pattern filter
    .build()?;
```

### Filter Decorators

Filter status by specific criteria:

```rust
use gixkit::StatusFilterIter;
use gixkit::filters::StatusFilter;

// Only files with changes
let changed = StatusFilterIter::new(
    status_iter,
    StatusFilter::Changed
);

// Only staged files
let staged = StatusFilterIter::new(
    status_iter,
    StatusFilter::Staged
);

// Only worktree-modified files
let worktree = StatusFilterIter::new(
    status_iter,
    StatusFilter::Worktree
);

// Specific status types
let added = StatusFilterIter::new(
    status_iter,
    StatusFilter::IndexStatus(StatusChar::Added)
);

let deleted = StatusFilterIter::new(
    status_iter,
    StatusFilter::WorktreeStatus(StatusChar::Deleted)
);

// Custom predicate
let custom = StatusFilterIter::new(
    status_iter,
    StatusFilter::Custom(|s| s.is_staged() && !s.is_worktree_modified())
);

for result in custom {
    let status = result?;
    // process only staged but not worktree-modified files
}
```

### Date Decorator

Add modification time and size information to status:

```rust
use gixkit::DateIter;

let dated_iter = DateIter::new(status_iter, work_dir);

for result in dated_iter {
    let file = result?;
    println!("{}: {:?} ({} bytes)",
        file.status.path,
        file.modified_time,
        file.size
    );
}
```

#### File Status With Date

```rust
pub struct FileStatusWithDate {
    pub status: FileStatus,
    pub modified_time: std::time::SystemTime,
    pub size: u64,
}
```

## Usage Examples

### For `got nah pick`

Get untracked files for interactive selection:

```rust
use gixkit::UntrackedIterBuilder;
use gixkit::UntrackedFilter;

pub fn get_untracked_files() -> Result<Vec<String>> {
    let repo = gix::open(".")?;

    let files: Vec<String> = UntrackedIterBuilder::new(&repo)
        .filter(UntrackedFilter::Normal) // Only top-level
        .build()?
        .filter_map(|r| r.ok())
        .map(|status| status.path)
        .collect();

    Ok(files)
}
```

### For `got statusd`

Get file status with modification times (oldest first):

```rust
use gixkit::{StatusIterBuilder, DateIter};
use gixkit::UntrackedFilter;

pub fn get_status_with_dates() -> Result<Vec<FileStatusWithDate>> {
    let repo = gix::open(".")?;
    let work_dir = repo.work_dir().unwrap().to_path_buf();

    let status_iter = StatusIterBuilder::new(&repo)
        .show_untracked(false)
        .build()?;

    let dated_iter = DateIter::new(status_iter, work_dir);

    let mut results: Vec<_> = dated_iter.collect::<Result<Vec<_>>>()?;
    results.sort_by_key(|f| f.modified_time);

    Ok(results)
}
```

### For `got goldest`

Find the oldest modified file:

```rust
use gixkit::{StatusIterBuilder, DateIter};

pub fn get_oldest_changed_file() -> Result<Option<FileStatusWithDate>> {
    let repo = gix::open(".")?;
    let work_dir = repo.work_dir().unwrap().to_path_buf();

    let status_iter = StatusIterBuilder::new(&repo)
        .show_untracked(false)
        .build()?;

    let dated_iter = DateIter::new(status_iter, work_dir);

    dated_iter
        .filter_map(|r| r.ok())
        .min_by_key(|f| f.modified_time)
        .map(Ok)
        .transpose()
}
```

### For Porcelain Binary

Display full git status in porcelain format:

```rust
use gixkit::{open_repo, StatusIterBuilder, UntrackedIterBuilder};

fn main() -> Result<()> {
    let repo = open_repo(".")?;

    // Iterate over tracked files
    let status_iter = StatusIterBuilder::new(&repo)
        .show_untracked(false) // Handle untracked separately
        .build()?;

    for result in status_iter {
        let status = result?;
        println!("{}{} {}",
            char::from(status.index_status),
            char::from(status.worktree_status),
            status.path
        );
    }

    // Iterate over untracked files
    let untracked_iter = UntrackedIterBuilder::new(&repo)
        .filter(UntrackedFilter::Normal)
        .build()?;

    for result in untracked_iter {
        let status = result?;
        println!("?? {}", status.path);
    }

    Ok(())
}
```

### Custom Filtering

Combine filters for specific use cases:

```rust
use gixkit::StatusIterBuilder;
use gixkit::filters::{StatusFilterIter, StatusFilter};

// Only staged added files
let staged_added = StatusFilterIter::new(
    StatusIterBuilder::new(&repo).build()?,
    StatusFilter::IndexStatus(StatusChar::Added)
);

// Only worktree-modified files
let worktree_modified = StatusFilterIter::new(
    StatusIterBuilder::new(&repo).build()?,
    StatusFilter::WorktreeStatus(StatusChar::Modified)
);

// Only modified but not staged
let modified_not_staged = StatusFilterIter::new(
    StatusIterBuilder::new(&repo).build()?,
    StatusFilter::Custom(|s| 
        s.is_worktree_modified() && !s.is_staged()
    )
);

// Collect results
let results: Vec<_> = modified_not_staged.collect::<Result<Vec<_>>>()?;
```

### Iterator Composition

Chain multiple operations:

```rust
use gixkit::{StatusIterBuilder, DateIter};
use gixkit::filters::StatusFilterIter;
use gixkit::filters::StatusFilter;

let results: Vec<_> = StatusIterBuilder::new(&repo)
    .build()?
    // Add date information
    .and_then(|status| {
        DateIter::new(std::iter::once(status), work_dir).next()
    })
    // Filter to only worktree modifications
    .filter(|result| {
        result.as_ref()
            .map(|f| f.status.is_worktree_modified())
            .unwrap_or(false)
    })
    .collect::<Result<Vec<_>>>()?;
```

## Design Decisions

### Lazy vs Eager

**Decision**: Lazy evaluation

**Rationale**: Large repositories can have thousands of files. Collecting all file statuses into a `Vec` before processing consumes significant memory. Lazy iteration allows processing files one at a time, with early termination support.

### Owned vs Borrowed

**Decision**: Borrowed from repository

**Rationale**: File paths and data are borrowed from the repository object with lifetime bounds. This avoids copying large path strings and OID arrays during iteration.

### Error Handling

**Decision**: Iterator yields `Result<Item>`

**Rationale**: Git operations can fail on a per-item basis (e.g., permission errors reading files). Yielding `Result` allows consumers to handle individual failures without aborting the entire iteration.

### Builder Pattern

**Decision**: Builder pattern for iterator configuration

**Rationale**: Iterators have many configuration options (filters, recursion levels, path patterns). A builder API provides clear, named configuration without dozens of constructor parameters.

### Composable Decorators

**Decision**: Separate filter and decorator types

**Rationale**: Filters and decorators can be combined using standard iterator combinators. This allows flexible composition without bloating the core iterator types.

## Module Structure

```
src/
├── lib.rs          # Public API surface
├── types.rs        # Core types (FileStatus, StatusChar, etc.)
├── repo.rs         # Repository operations (open_repo, get_head_tree)
├── index.rs        # Index operations (get_index_entries, path_in_index)
├── status.rs       # StatusIter implementation
├── untracked.rs    # UntrackedIter implementation
├── filters.rs      # Filter decorators
├── decorators.rs   # Utility decorators (DateIter)
└── examples.rs     # Example code (used in documentation)
```

## Performance Considerations

### Memory Usage

Iterators in `gixkit` are designed to be memory-efficient:

- **Index iteration**: Uses gix's lazy index entry iteration
- **Tree lookups**: Reuses buffer across iterations
- **Worktree scanning**: Iterates directory-by-directory without loading entire tree

### Caching

Repository state is cached in iterator structs:

- HEAD tree computed once at construction
- Index opened once at construction
- Working directory path cached

### Future Optimizations

Potential improvements:

- Parallel directory scanning for large repos
- Cached OID computations for repeated operations
- Lazy metadata fetching for untracked files

## Testing

Run tests with:

```bash
cargo test
```

Tests cover:

- Repository operations (opening, HEAD tree retrieval)
- Index operations (entry iteration, path lookup)
- Status computation (staged, unstaged, untracked)
- Filtering (various filter types)
- Decorator composition

## License

MIT OR Apache-2.0

## Contributing

When adding new iterator types:

1. Follow builder pattern for configuration
2. Implement `Iterator` trait yielding `Result<Item>`
3. Cache repository state in struct fields
4. Add comprehensive documentation with examples
5. Include unit tests for all code paths

## Future Enhancements

Planned features:

- **Pathspec support**: Filter by git-style path patterns
- **Ignore file parsing**: Respect `.gitignore` rules in untracked iteration
- **Submodule handling**: Proper status for git submodules
- **Merge conflict detection**: Detailed status for unmerged files
- **Binary file detection**: Identify binary vs text files
- **Parallel iteration**: `par_iter` support for large repos
