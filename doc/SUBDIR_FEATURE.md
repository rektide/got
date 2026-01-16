# RepoIter Subdirectory Feature + UntrackedIter Comparison

## What is `work_dir`?

`work_dir` is the **repository root** - the working directory of the git repository.

```rust
let work_dir = repo.work_dir()?;  
// Example: /home/rektide/src/got
```

**Purpose:**
- Convert absolute filesystem paths to relative repo paths
- Strip repo root prefix from paths (e.g., `/home/rektide/src/got/crate/gixkit/src/status.rs` → `crate/gixkit/src/status.rs`)
- Compute full paths for file reading

## New Feature: Subdirectory Iteration

**Goal:** Iterate only within a subdirectory subtree of the repository.

**Example:**
```rust
// Iterate only files in crate/gixkit/src/
RepoIter::builder(Arc::clone(&repo))
    .subdir("crate/gixkit/src")  // NEW: limit to subtree
    .mode(IterMode::Untracked)
    .build()?
```

**What this means:**
1. Don't build state for files outside the subtree
2. Don't return files outside the subtree during iteration
3. Optimize: skip index entries not in subtree
4. Optimize: skip filesystem traversal outside subtree

## Subdirectory Feature Implementation

### Builder API

```rust
pub struct RepoIterBuilder {
    repo: Arc<Repository>,
    mode: IterMode,
    status_filter: Option<Vec<StatusChar>>,
    include_metadata: bool,
    subdir: Option<PathBuf>,  // NEW: optional subtree
}
```

```rust
pub fn subdir(mut self, path: impl AsRef<Path>) -> Self {
    self.subdir = Some(path.as_ref().to_path_buf());
    self
}
```

### Pre-computation Optimizations

**Skip irrelevant index entries:**
```rust
fn new(repo: Arc<Repository>, ...) -> Result<Self> {
    let work_dir = repo.work_dir()?.to_path_buf();
    
    let index_entries: Vec<(BString, ObjectId)> = if let Some(ref subdir) = subdir {
        // Only include index entries matching subdir
        index.entries()
            .filter(|entry| entry.path_in_subdir(&subdir))
            .map(|entry| (entry.path.to_owned(), entry.id))
            .collect()
    } else {
        // Include all index entries
        index.entries()
            .map(|entry| (entry.path.to_owned(), entry.id))
            .collect()
    };
    
    let index_set: HashSet<BString> = index_entries
        .iter()
        .map(|(path, _)| path.clone())
        .collect();
}
```

**Filesystem traversal:**
```rust
impl RepoIter {
    fn new(repo: Arc<Repository>, builder: RepoIterBuilder) -> Result<Self> {
        let work_dir = repo.work_dir()?.to_path_buf();
        
        // Start traversal from subdir if specified, else work_dir
        let start_dir = if let Some(ref subdir) = builder.subdir {
            work_dir.join(subdir)
        } else {
            work_dir.clone()
        };
        
        let dir_stack = if needs_untracked {
            vec![start_dir]
        } else {
            vec![]
        };
        
        Ok(Self {
            repo,
            work_dir,
            subdir: builder.subdir,
            dir_stack,
            // ...
        })
    }
}
```

### Iteration Filtering

**Skip files outside subtree:**
```rust
fn next_tracked(&mut self) -> Option<Result<FileStatus>> {
    while let Some((path, oid)) = self.tracked_iter.next() {
        let file_status = FileStatus { ... };
        
        // Skip if outside subdir (already filtered in pre-computation,
        // but keep as safety check)
        if let Some(ref subdir) = self.subdir {
            if !path_str.starts_with(&subdir.to_string_lossy()) {
                continue;
            }
        }
        
        return Some(Ok(file_status));
    }
    None
}
```

**For untracked traversal, we already start in subdir so this is automatic.**

## UntrackedIter: Old vs New

### What UntrackedIter Used to Do

```rust
pub struct UntrackedIter<'repo> {
    repo: &'repo Repository,
    work_dir: PathBuf,
    dir_stack: Vec<PathBuf>,
    current_dir_iter: Option<std::fs::ReadDir>,
    filter: UntrackedFilter,
}

fn new(...) {
    let work_dir = repo.work_dir()?.to_path_buf();
    let dir_stack = vec![work_dir];  // Always start at repo root
}

fn next(&mut self) {
    loop {
        // Get next directory to explore
        if self.current_dir_iter.is_none() {
            let dir = self.dir_stack.pop()?;
            self.current_dir_iter = Some(std::fs::read_dir(&dir)?);
        }
        
        // Process entries in current directory
        for entry in self.current_dir_iter {
            // Skip hidden files and .git
            if file_name.starts_with('.') || file_name == ".git" {
                continue;
            }
            
            // Get relative path
            let rel_path = path.strip_prefix(&self.work_dir)?;
            
            // ❌ O(n×m) - scan ALL index entries!
            if self.path_is_tracked(rel_path) {
                continue;
            }
            
            // If directory and filter==All, add to stack
            if path.is_dir() && self.filter == UntrackedFilter::All {
                self.dir_stack.push(path);
            }
            
            yield FileStatus { ... };
        }
    }
}
```

### What RepoIter Does Differently

```rust
pub struct RepoIter {
    repo: Arc<Repository>,
    work_dir: PathBuf,
    subdir: Option<PathBuf>,  // NEW: configurable subtree
    dir_stack: Vec<PathBuf>,
    current_dir_iter: Option<std::fs::ReadDir>,
    mode: IterMode,
    
    // ✅ NEW: Pre-computed for O(1) lookups
    index_set: HashSet<BString>,
}

fn new(...) {
    let work_dir = repo.work_dir()?.to_path_buf();
    
    // ✅ NEW: Start from subdir if configured
    let start_dir = if let Some(ref subdir) = builder.subdir {
        work_dir.join(subdir)
    } else {
        work_dir.clone()
    };
    
    let dir_stack = vec![start_dir];
    
    // ✅ NEW: Pre-compute index set for O(1) lookups
    let index_set: HashSet<BString> = build_index_set(&repo, &subdir)?;
}

fn next(&mut self) {
    loop {
        // Same directory traversal pattern
        if self.current_dir_iter.is_none() {
            let dir = self.dir_stack.pop()?;
            self.current_dir_iter = Some(std::fs::read_dir(&dir)?);
        }
        
        for entry in self.current_dir_iter {
            // Skip hidden files and .git (same)
            if file_name.starts_with('.') || file_name == ".git" {
                continue;
            }
            
            // Get relative path (same)
            let rel_path = path.strip_prefix(&self.work_dir)?;
            
            // ✅ O(1) lookup instead of O(n×m)!
            if self.index_set.contains(rel_path_bstr) {
                continue;
            }
            
            // If directory, add to stack (same)
            if path.is_dir() && self.mode.allows_untracked_dirs() {
                self.dir_stack.push(path);
            }
            
            yield FileStatus { ... };
        }
    }
}
```

## Key Differences

| Aspect | UntrackedIter (Old) | RepoIter (New) |
|--------|---------------------|-----------------|
| **Starting point** | Always `work_dir` (repo root) | `work_dir` or `subdir` (configurable) |
| **Index lookup** | `path_is_tracked()`: O(n×m) full scan | `index_set.contains()`: O(1) lookup |
| **State at construction** | Just `work_dir` | Pre-computed `index_set` (HashSet) |
| **Directory traversal** | Manual dir_stack + ReadDir | Same pattern (proven effective) |
| **Filtering tracked files** | Linear scan per file | Pre-filtered + HashSet lookup |
| **Performance** | Poor for large repos | Optimal (O(n) instead of O(n×m)) |
| **Subdirectory support** | None (always repo-wide) | ✅ Yes via `subdir()` option |

## Example Performance Comparison

**Scenario:** Repository with 10,000 index entries, 1,000 untracked files

**UntrackedIter (Old):**
```
For each untracked file:
  - Scan 10,000 index entries to check if tracked
Total operations: 1,000 × 10,000 = 10,000,000 comparisons
```

**RepoIter (New):**
```
At construction:
  - Build HashSet: 10,000 insertions

During iteration:
  For each untracked file:
    - O(1) HashSet lookup
Total operations: 10,000 + 1,000 = 11,000 operations

Speedup: ~900x faster
```

## Subdirectory Performance Impact

**Without subdir:**
- Pre-compute index set with all entries
- Walk entire filesystem

**With subdir `crate/gixkit/src/`:**
- Pre-compute index set with only matching entries (filtered)
- Walk only `crate/gixkit/src/` subtree
- Skip all other files during iteration

**Impact:**
- Less memory (smaller index set)
- Faster construction (fewer HashSet insertions)
- Faster iteration (skip irrelevant files)
- Especially useful for large repos when working in small subdirectory

## Summary

**Work_dir** = Repository root (unchanged)
**Subdir** = Optional subtree (NEW feature)

**RepoIter improvements over UntrackedIter:**
1. ✅ Pre-computed `index_set` for O(1) lookups (massive performance win)
2. ✅ Subdirectory support for scoped iteration
3. ✅ Same proven directory traversal pattern
4. ✅ Eliminates O(n×m) bottleneck
