# RepoIter Design Document

## Goals

1. Merge `StatusIter` + `UntrackedIter` + `StatusFilterIter` + `DateIter` into single `RepoIter`
2. All configuration via builder pattern
3. Lazy computation at iteration time
4. Pre-compute shared state (index set) for efficiency
5. Support both `FileStatus` and `FileStatusWithDate` output

## Design

### Builder Configuration

```rust
pub struct RepoIterBuilder {
    repo: Arc<Repository>,
    mode: IterMode,
    status_filter: Option<Vec<StatusChar>>,
    include_metadata: bool,
}

pub enum IterMode {
    Tracked,      // Only tracked files (from index)
    Untracked,    // Only untracked files (filesystem walk)
    Both,         // Tracked then untracked files
}
```

**Usage:**
```rust
// Tracked files only
RepoIter::builder(Arc::clone(&repo))
    .mode(IterMode::Tracked)
    .build()?

// Untracked files only
RepoIter::builder(Arc::clone(&repo))
    .mode(IterMode::Untracked)
    .build()?

// All files (tracked + untracked)
RepoIter::builder(Arc::clone(&repo))
    .mode(IterMode::Both)
    .build()?

// Filter by status
RepoIter::builder(Arc::clone(&repo))
    .mode(IterMode::Both)
    .filter(vec![StatusChar::Added, StatusChar::Modified])
    .build()?

// Include metadata (like DateIter)
RepoIter::builder(Arc::clone(&repo))
    .mode(IterMode::Both)
    .include_metadata(true)
    .build()?
```

### Internal State

```rust
pub struct RepoIter {
    repo: Arc<Repository>,
    work_dir: PathBuf,
    head_tree_id: ObjectId,
    
    // Shared pre-computed state
    index_entries: Vec<(BString, ObjectId)>,
    index_set: HashSet<BString>,
    
    // Tracked file iteration
    tracked_iter: std::vec::IntoIter<(BString, ObjectId)>,
    
    // Untracked file iteration
    untracked_dir_stack: Vec<PathBuf>,
    untracked_current_iter: Option<std::fs::ReadDir>,
    
    // Configuration
    mode: IterMode,
    status_filter: Option<Vec<StatusChar>>,
    include_metadata: bool,
    
    // Internal state
    phase: IterationPhase,  // Tracks whether we're in tracked/untracked phase
}
```

```rust
enum IterationPhase {
    Tracked,
    Untracked,
    Done,
}
```

### Output Types

**Two output variants based on `include_metadata`:**

```rust
impl Iterator for RepoIter {
    // If include_metadata == false:
    type Item = Result<FileStatus>;
    
    // If include_metadata == true:
    // type Item = Result<FileStatusWithDate>;
}
```

**Problem:** Can't have conditional `type Item` in Rust.

**Solutions:**

**Option A: Enum output**
```rust
pub enum RepoItem {
    Status(FileStatus),
    StatusWithDate(FileStatusWithDate),
}
```

**Pros:** Single iterator type
**Cons:** Consumer must match on enum

**Option B: Two separate iterators**
```rust
impl RepoIter {
    pub fn files(&mut self) -> impl Iterator<Item = Result<FileStatus>> + '_ { ... }
    pub fn files_with_metadata(&mut self) -> impl Iterator<Item = Result<FileStatusWithDate>> + '_ { ... }
}
```

**Pros:** Clear type at call site
**Cons:** Can't call both on same iterator

**Option C: Generic iterator type**
```rust
pub struct RepoIter<T = FileStatus> { ... }

impl Iterator for RepoIter<FileStatus> {
    type Item = Result<FileStatus>;
}

impl Iterator for RepoIter<FileStatusWithDate> {
    type Item = Result<FileStatusWithDate>;
}

impl RepoIter<FileStatus> {
    pub fn with_metadata(self) -> RepoIter<FileStatusWithDate> { ... }
}
```

**Pros:** Type-safe, can convert
**Cons:** More complex generics

**Option D: Always compute metadata, consumer ignores if unwanted**
```rust
// Always return FileStatusWithDate
type Item = Result<FileStatusWithDate>;

// Consumer can ignore metadata if they don't want it:
for result in iter {
    let file_with_date = result?;
    // Just use file_with_date.status
}
```

**Pros:** Simple, single type
**Cons:** Wastes computation if metadata not needed

**Recommendation: Option B (Two separate methods)**

**Why:**
- Clear intent at call site
- No computation wasted (lazy as requested)
- Type-safe
- Can create new iterator with same config if needed for both

### Iteration Logic

**Phase 1: Tracked files**
```rust
fn next_tracked(&mut self) -> Option<Result<FileStatus>> {
    while let Some((path, oid)) = self.tracked_iter.next() {
        // Compute status
        let (index_status, worktree_status) = self.compute_status(&path, oid);
        
        let file_status = FileStatus {
            path: path.to_string(),
            index_status: StatusChar::from_char(index_status),
            worktree_status: StatusChar::from_char(worktree_status),
        };
        
        // Filter by status if configured
        if let Some(ref filter) = self.status_filter {
            if !filter.contains(&file_status.index_status) &&
               !filter.contains(&file_status.worktree_status) {
                continue;
            }
        }
        
        // Only return files with changes (like original StatusIter)
        if file_status.has_changes() {
            return Some(Ok(file_status));
        }
    }
    None
}
```

**Phase 2: Untracked files**
```rust
fn next_untracked(&mut self) -> Option<Result<FileStatus>> {
    // Directory traversal with O(1) index lookup
    while let Some(entry_result) = self.next_dir_entry() {
        if !entry.is_file() {
            continue;
        }
        
        let rel_path = entry.path().strip_prefix(&self.work_dir)?;
        let rel_path_bstr = BStr::new(rel_path.to_str()?);
        
        // O(1) lookup instead of O(n×m) scan!
        if self.index_set.contains(rel_path_bstr) {
            continue;  // Skip tracked files
        }
        
        return Some(Ok(FileStatus {
            path: rel_path.to_string(),
            index_status: StatusChar::None,
            worktree_status: StatusChar::Untracked,
        }));
    }
    None
}
```

**Combined iteration:**
```rust
fn next(&mut self) -> Option<Self::Item> {
    match self.phase {
        IterationPhase::Tracked => {
            if let Some(result) = self.next_tracked() {
                return Some(self.maybe_add_metadata(result));
            }
            self.phase = IterationPhase::Untracked;
            self.next()
        }
        IterationPhase::Untracked => {
            if let Some(result) = self.next_untracked() {
                return Some(self.maybe_add_metadata(result));
            }
            self.phase = IterationPhase::Done;
            None
        }
        IterationPhase::Done => None,
    }
}
```

### Lazy Metadata Computation

**Only compute when `include_metadata` is true:**

```rust
fn maybe_add_metadata(&mut self, file_status: FileStatus) -> Self::Item {
    if !self.include_metadata {
        return Ok(file_status);
    }
    
    let full_path = self.work_dir.join(&file_status.path);
    
    let metadata = std::fs::metadata(&full_path)?;
    let modified_time = metadata.modified()?;
    let size = metadata.len();
    
    Ok(FileStatusWithDate {
        status: file_status,
        modified_time,
        size,
    })
}
```

**Note:** If file was deleted, metadata will fail - we should handle that:

```rust
fn maybe_add_metadata(&mut self, file_status: FileStatus) -> Self::Item {
    if !self.include_metadata {
        return Ok(file_status);
    }
    
    // If file was deleted, we can't get metadata
    if file_status.worktree_status == StatusChar::Deleted {
        return Ok(FileStatusWithDate {
            status: file_status,
            modified_time: std::time::SystemTime::UNIX_EPOCH,
            size: 0,
        });
    }
    
    let full_path = self.work_dir.join(&file_status.path);
    let metadata = std::fs::metadata(&full_path)?;
    
    Ok(FileStatusWithDate {
        status: file_status,
        modified_time: metadata.modified()?,
        size: metadata.len(),
    })
}
```

## Efficiency Improvements

### 1. Pre-computed Index HashSet

**Old (UntrackedIter):** O(n×m) where n=untracked files, m=index entries
```rust
for entry in index.entries() {  // Scan ALL entries per file
    if entry.path(&index) == path {
        return true;
    }
}
```

**New (RepoIter):** O(n) where n=untracked files
```rust
self.index_set.contains(path)  // O(1) lookup
```

### 2. Single Filesystem Walk

**Old:**
```
StatusIter: Walk index (1 pass through tracked files)
UntrackedIter: Walk filesystem (1 pass)
```

**New:**
```
RepoIter(Both): Walk index + filesystem (1 combined pass)
```

### 3. Optional Single File Read for Status + Metadata

**Old:**
```
StatusIter: Read file for hash computation
DateIter: Read file metadata (stat syscall)
```

**New:**
```
RepoIter(include_metadata): Read file once for hash + metadata
```

**Implementation:**
```rust
fn compute_status_and_metadata(&self, path: &BString, oid: ObjectId) 
    -> (char, char, Option<(SystemTime, u64)>)
{
    // ... compute index status ...
    
    let full_path = self.work_dir.join(&path.to_string());
    let metadata_opt = std::fs::metadata(&full_path).ok();
    
    let worktree_status = if let Some(metadata) = metadata_opt {
        let content = std::fs::read(&full_path)?;
        let file_hash = gix_object::compute_hash(..., &content);
        
        if file_hash != oid {
            'M'
        } else {
            ' '
        }
    } else {
        'D'  // Deleted
    };
    
    (index_status, worktree_status, metadata_opt.map(|m| (m.modified().unwrap(), m.len())))
}
```

## Implementation Plan

1. Create `RepoIter` and `RepoIterBuilder` in new file
2. Implement builder methods:
   - `mode(IterMode)`
   - `filter(Vec<StatusChar>)`
   - `include_metadata(bool)`
3. Implement iterator with lazy computation
4. Pre-compute index entries + HashSet in construction
5. Implement tracked file iteration (from StatusIter)
6. Implement untracked file iteration (from UntrackedIter, with HashSet)
7. Combine in single iterator with phase tracking
8. Implement lazy metadata computation
9. Add tests
10. Update consumers to use RepoIter
11. Deprecate old iterators (or remove after migration)

## Migration Examples

### Before (Multiple iterators)
```rust
let repo = open_repo(".")?;
let repo = Arc::new(repo);

// Track changes
let status_iter = StatusIter::builder(Arc::clone(&repo)).build()?;
for file in status_iter {
    println!("M {}", file?.path);
}

// Track untracked
let untracked_iter = UntrackedIter::builder(Arc::clone(&repo)).build()?;
for file in untracked_iter {
    println!("?? {}", file?.path);
}

// Filter
let filtered = StatusFilterIter::new(status_iter, StatusFilter::Changed);

// Add metadata
let dated = DateIter::new(filtered, work_dir);
```

### After (Single iterator)
```rust
let repo = open_repo(".")?;
let repo = Arc::new(repo);

// All in one pass
let iter = RepoIter::builder(Arc::clone(&repo))
    .mode(IterMode::Both)
    .build()?;

for file in iter {
    let file = file?;
    match (file.index_status, file.worktree_status) {
        (StatusChar::None, StatusChar::Untracked) => println!("?? {}", file.path),
        (StatusChar::Modified, _) => println!("M {}", file.path),
        (StatusChar::Added, _) => println!("A {}", file.path),
        _ => {}
    }
}
```

### With metadata
```rust
let iter = RepoIter::builder(Arc::clone(&repo))
    .mode(IterMode::Both)
    .include_metadata(true)
    .build()?;

for file in iter {
    let file = file?;
    let modified = chrono::DateTime::<Utc>::from(file.modified_time)
        .format("%Y-%m-%d").to_string();
    println!("{} {} {}", modified, file.size, file.path);
}
```
