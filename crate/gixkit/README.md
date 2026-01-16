# gixkit

Git iterator library for `got` toolkit.

## Overview

`gixkit` provides a unified, lazy iterator for git operations built on top of the `gix` library. It centralizes repository handling, index iteration, tree lookups, and status computation into a single, well-tested library.

### Key Features

- **Unified iterator**: Single `RepoIter` handles tracked/untracked files with flexible configuration
- **Lazy evaluation**: Avoids loading entire repository state into memory
- **Builder pattern**: Flexible configuration (mode, filters, metadata, subdirectory)
- **Arc<Repository>**: Shared ownership without lifetime complexity
- **Error-per-item**: Individual iteration failures don't abort entire operation

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
    pub metadata: Option<FileMetadata>,
}

pub struct FileMetadata {
    pub modified_time: std::time::SystemTime,
    pub size: u64,
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

### Iteration Mode

Controls which files to iterate over:

```rust
use gixkit::IterMode;

pub enum IterMode {
    Tracked,     // Only tracked files
    Untracked,   // Only untracked files
    Both,        // Both tracked and untracked files
}
```

## API Reference

### Repository Operations

#### Opening Repositories

```rust
use gixkit::open_repo;
use std::sync::Arc;

let repo = Arc::new(open_repo(".")?);

// With explicit path
let repo = Arc::new(open_repo("/path/to/repo")?);
```

#### Getting HEAD Tree

```rust
use gixkit::get_head_tree;

let head_tree = get_head_tree(&repo)?;
// Returns empty tree for new repositories
```

### RepoIter

Unified iterator for tracking file status:

```rust
use gixkit::{RepoIterBuilder, IterMode};
use std::sync::Arc;

let iter = RepoIterBuilder::new(Arc::clone(&repo))
    .mode(IterMode::Both)
    .include_metadata(true)
    .subdir("src")
    .build()?;

for result in iter {
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
RepoIterBuilder::new(Arc::clone(&repo))
    .mode(IterMode::Both)              // Tracked | Untracked | Both
    .filter(vec![StatusChar::Modified]) // Filter by status types
    .include_metadata(true)             // Include file metadata
    .subdir("src")                     // Limit to subdirectory
    .build()?;
```

#### Iteration Modes

```rust
// Only tracked files (index entries)
let tracked = RepoIterBuilder::new(Arc::clone(&repo))
    .mode(IterMode::Tracked)
    .build()?;

// Only untracked files
let untracked = RepoIterBuilder::new(Arc::clone(&repo))
    .mode(IterMode::Untracked)
    .build()?;

// Both tracked and untracked files
let both = RepoIterBuilder::new(Arc::clone(&repo))
    .mode(IterMode::Both)
    .build()?;
```

#### Status Filtering

```rust
// Only modified files
let modified = RepoIterBuilder::new(Arc::clone(&repo))
    .mode(IterMode::Both)
    .filter(vec![StatusChar::Modified])
    .build()?;

// Only staged files
let staged = RepoIterBuilder::new(Arc::clone(&repo))
    .mode(IterMode::Both)
    .filter(vec![StatusChar::Added, StatusChar::Modified])
    .build()?;

// Custom filtering
let iter = RepoIterBuilder::new(Arc::clone(&repo))
    .mode(IterMode::Both)
    .build()?
    .filter(|r| {
        r.as_ref()
            .map(|s| s.is_staged())
            .unwrap_or(false)
    });
```

#### Subdirectory Support

```rust
// Limit iteration to a specific subdirectory
let subdir_iter = RepoIterBuilder::new(Arc::clone(&repo))
    .mode(IterMode::Both)
    .subdir("src/components")
    .include_metadata(true)
    .build()?;

// This reduces memory and computation for large repositories
// when you only care about files in a specific directory
```

## Usage Examples

### For `got nah pick`

Get untracked files for interactive selection:

```rust
use gixkit::{RepoIterBuilder, IterMode};
use std::sync::Arc;

pub fn get_untracked_files() -> Result<Vec<String>> {
    let repo = Arc::new(gixkit::open_repo(".")?);

    let files: Vec<String> = RepoIterBuilder::new(repo)
        .mode(IterMode::Untracked)
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
use gixkit::{RepoIterBuilder, IterMode};
use std::sync::Arc;

pub fn get_status_with_dates() -> Result<Vec<gixkit::FileStatus>> {
    let repo = Arc::new(gixkit::open_repo(".")?);

    let mut results: Vec<_> = RepoIterBuilder::new(Arc::clone(&repo))
        .mode(IterMode::Tracked)
        .include_metadata(true)
        .build()?
        .collect::<Result<Vec<_>>>()?;

    results.sort_by_key(|f| {
        f.metadata
            .as_ref()
            .map(|m| m.modified_time)
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });

    Ok(results)
}
```

### For `got goldest`

Find oldest modified file:

```rust
use gixkit::{RepoIterBuilder, IterMode};
use std::sync::Arc;

pub fn get_oldest_changed_file() -> Result<Option<gixkit::FileStatus>> {
    let repo = Arc::new(gixkit::open_repo(".")?);

    let mut results: Vec<_> = RepoIterBuilder::new(Arc::clone(&repo))
        .mode(IterMode::Tracked)
        .include_metadata(true)
        .build()?
        .collect::<Result<Vec<_>>>()?;

    results.sort_by_key(|f| {
        f.metadata
            .as_ref()
            .map(|m| m.modified_time)
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });

    Ok(results.into_iter().next())
}
```

### For Porcelain Binary

Display full git status in porcelain format:

```rust
use gixkit::{open_repo, RepoIterBuilder, IterMode};
use std::sync::Arc;

fn main() -> Result<()> {
    let repo = Arc::new(open_repo(".")?);

    let iter = RepoIterBuilder::new(Arc::clone(&repo))
        .mode(IterMode::Both)
        .build()?;

    for result in iter {
        let status = result?;
        let index_char: char = status.index_status.into();
        let worktree_char: char = status.worktree_status.into();

        match (status.index_status, status.worktree_status) {
            (StatusChar::Untracked, StatusChar::Untracked) => {
                println!("?? {}", status.path);
            }
            _ => {
                println!("{}{} {}",
                    index_char,
                    worktree_char,
                    status.path
                );
            }
        }
    }

    Ok(())
}
```

### Subdirectory Iteration

Process only files in a specific directory:

```rust
use gixkit::{RepoIterBuilder, IterMode};
use std::sync::Arc;

let repo = Arc::new(open_repo(".")?);

let iter = RepoIterBuilder::new(Arc::clone(&repo))
    .mode(IterMode::Both)
    .include_metadata(true)
    .subdir("src")
    .build()?;

for result in iter {
    let status = result?;
    if let Some(metadata) = status.metadata {
        println!("{} ({} bytes)", status.path, metadata.size);
    }
}
```

### Custom Filtering

Combine mode and filtering for specific use cases:

```rust
use gixkit::{RepoIterBuilder, IterMode, StatusChar};

// Only staged added files
let staged_added = RepoIterBuilder::new(Arc::clone(&repo))
    .mode(IterMode::Both)
    .filter(vec![StatusChar::Added])
    .build()?;

// Only worktree-modified files
let worktree_modified = RepoIterBuilder::new(Arc::clone(&repo))
    .mode(IterMode::Both)
    .filter(vec![StatusChar::Modified])
    .build()?;

// Custom filter with iterator combinators
let modified_not_staged = RepoIterBuilder::new(Arc::clone(&repo))
    .mode(IterMode::Both)
    .build()?
    .filter(|result| {
        result.as_ref()
            .map(|s| s.is_worktree_modified() && !s.is_staged())
            .unwrap_or(false)
    });

// Collect results
let results: Vec<_> = modified_not_staged.collect::<Result<Vec<_>>>()?;
```

## Design Decisions

### Unified Iterator

**Decision**: Single `RepoIter` instead of separate StatusIter and UntrackedIter

**Rationale**: Maintaining multiple iterators for similar functionality leads to code duplication and performance issues (e.g., UntrackedIter's O(n×m) index lookups). A unified iterator provides:
- Consistent API across all use cases
- Eliminates redundant filesystem operations
- Single-pass iteration with phase-based design
- Shared metadata computation logic

### Arc<Repository> vs Owned

**Decision**: Arc<Repository> shared ownership

**Rationale**: Using Arc provides:
- No lifetime complexity (Arc::clone is just a pointer copy + refcount bump)
- Consistency across all iterators
- Cheap to clone (O(1))
- Allows storing iterators in structs without lifetime concerns

### Lazy Metadata Computation

**Decision**: `include_metadata` flag instead of always computing metadata

**Rationale**: Metadata computation requires filesystem I/O:
- Many consumers don't need metadata (e.g., simple status display)
- Lazy computation reduces unnecessary filesystem reads
- Consumers opt-in when needed via `include_metadata(true)`

### Subdirectory Filtering

**Decision**: Subdirectory constraint as builder option

**Rationale**: For large repositories:
- Reduces memory usage (filter index entries before loading)
- Reduces computation (skip irrelevant directories)
- Useful for commands working in small subtree

### O(log n) Index Lookups

**Decision**: Use gix's `index.entry_by_path()` instead of pre-built HashSet

**Rationale**: 
- gix provides O(log n) lookups via index.entry_by_path()
- Avoids ~7MB memory overhead for 100k-entry repos
- Simpler code (no HashSet construction/maintenance)

### Error Handling

**Decision**: Iterator yields `Result<Item>`

**Rationale**: Git operations can fail on a per-item basis (e.g., permission errors reading files). Yielding `Result` allows consumers to handle individual failures without aborting entire iteration.

### Builder Pattern

**Decision**: Builder pattern for iterator configuration

**Rationale**: RepoIter has many configuration options (mode, filters, metadata, subdir). A builder API provides clear, named configuration without dozens of constructor parameters.

## Module Structure

```
src/
├── lib.rs          # Public API surface
├── types.rs        # Core types (FileStatus, StatusChar, FileMetadata)
├── repo.rs         # Repository operations (open_repo, get_head_tree)
├── repo_iter.rs    # Unified RepoIter implementation
└── filters.rs      # Filter utilities (StatusFilter, etc.)
```

## Performance Considerations

### Memory Usage

`RepoIter` is designed to be memory-efficient:

- **Index iteration**: Filters entries to vector once at construction
- **Tree lookups**: Reuses buffer across iterations
- **Worktree scanning**: Iterates directory-by-directory without loading entire tree
- **Metadata**: Lazy computation, only when `include_metadata=true`

### Caching

Repository state is cached in iterator structs:

- HEAD tree computed once at construction
- Index opened on-demand with caching
- Working directory path cached

### Index Lookup Performance

Uses gix's built-in `index.entry_by_path()` with O(log n) lookups:
- Avoids HashSet overhead (~7MB for 100k entries)
- No need to pre-compute/tracked sets
- Fast enough for file iteration patterns

### Subdirectory Optimization

When `subdir()` is specified:
- Index entries filtered before iteration
- Worktree traversal limited to subtree
- Significant memory savings for large repos

### Future Optimizations

Potential improvements:

- Parallel directory scanning for large repos
- Cached OID computations for repeated operations
- Streaming index entries for very large repos

## Testing

Run tests with:

```bash
cargo test
```

Tests cover:

- Repository operations (opening, HEAD tree retrieval)
- Index operations (entry iteration, path lookup)
- Status computation (staged, unstaged, untracked)
- Iteration modes (tracked, untracked, both)
- Metadata computation
- Subdirectory filtering

## License

MIT OR Apache-2.0

## Contributing

When adding features to RepoIter:

1. Follow builder pattern for configuration
2. Implement `Iterator` trait yielding `Result<Item>`
3. Cache repository state in struct fields
4. Add comprehensive documentation with examples
5. Include unit tests for all code paths
6. Consider subdirectory optimization for new features

## Future Enhancements

Planned features:

- **Pathspec support**: Filter by git-style path patterns
- **Ignore file parsing**: Respect `.gitignore` rules in untracked iteration
- **Submodule handling**: Proper status for git submodules
- **Merge conflict detection**: Detailed status for unmerged files
- **Binary file detection**: Identify binary vs text files
- **Parallel iteration**: `par_iter` support for large repos
