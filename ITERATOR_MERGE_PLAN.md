# Iterator Analysis for gixkit

## Current Iterators

| Iterator | Output Type | Computation (Construction) | Computation (Per File) | Notes |
|----------|-------------|---------------------------|------------------------|-------|
| `StatusIter` | `Result<FileStatus>` | - Extract all index entries once<br>- Get head tree OID | - `repo.find_tree(head_tree_id)`<br>- `tree.lookup_entry(path)`<br>- `entry.object()`<br>- Compare OIDs (index status)<br>- `std::fs::read(file)`<br>- Compute hash (worktree status) | Only outputs files with changes (`has_changes()` check)<br>- **Per-file tree lookup and file read** |
| `UntrackedIter` | `Result<FileStatus>`<br>(`index_status=None, worktree_status=Untracked`) | - Get work_dir path | - Directory traversal (dir_stack + ReadDir)<br>- **`path_is_tracked()`**: iterate through ALL index entries for each file<br>- Filter out `.git` and dot files | **O(n×m)** where n=untracked files, m=index entries<br>- **No shared index lookup - duplicated per file** |
| `StatusFilterIter` | `Result<FileStatus>` | None (just stores filter) | - No repo/fs lookups<br>- Calls predicates on FileStatus | Wrapper decorator - delegates to inner |
| `DateIter` | `Result<FileStatusWithDate>` | None (stores work_dir) | - `std::fs::metadata(path)`<br>- Extract `modified_time` and `size` | Wrapper decorator - **additional filesystem lookup per file** |

## Key Observations

### 1. Inefficient Index Lookup in UntrackedIter

**Problem:** `path_is_tracked()` iterates through ALL index entries for each untracked file:

```rust
fn path_is_tracked(&self, path: &BStr) -> bool {
    let index = self.repo.index()?;
    for entry in index.entries() {  // ❌ Full scan for each file!
        if entry.path(&index) == path {
            return true;
        }
    }
    false
}
```

**Impact:** If you have 10,000 index entries and 1,000 untracked files, that's 10,000,000 comparisons.

**Solution:** Use a `HashSet<BString>` for O(1) lookups (pre-computed during construction).

---

### 2. Redundant Filesystem Operations

**StatusIter** already reads the file to compute worktree hash:
```rust
let content = std::fs::read(&full_path)?;
let oid = gix_object::compute_hash(..., &content);
```

**DateIter** then reads the same file's metadata:
```rust
let metadata = std::fs::metadata(&full_path)?;
```

**Problem:** Two filesystem operations per file (could be optimized to one).

---

### 3. Separate Iterators Require Multiple Passes

**Current pattern:**
```rust
// First pass: iterate all index entries
for file in StatusIter::builder(Arc::clone(&repo)).build()? { ... }

// Second pass: walk entire filesystem
for file in UntrackedIter::builder(Arc::clone(&repo)).build()? { ... }
```

**Impact:**
- StatusIter reads all files from index (that changed)
- UntrackedIter walks entire filesystem again
- Each untracked file checks if tracked (full index scan)
- Decorators add more passes (DateIter reads metadata again)

---

### 4. Decorators Don't Re-lookup Data

**Good news:** Decorators are efficient wrappers:
- `StatusFilterIter`: No extra lookups, just predicates on `FileStatus`
- `DateIter`: Adds metadata lookup (see #2 above)

---

## Design Options

### Option A: Current Separate Iterators (What we have)

**Pros:**
- Clear separation of concerns
- Each iterator is focused
- Can use either/both independently

**Cons:**
- Multiple passes over data
- Inefficient index lookups in UntrackedIter
- Harder to share pre-computed data

**When to use:** Simple scripts, not concerned about performance

---

### Option B: Merged Iterator with State Filter

**Single iterator that handles both tracked and untracked files:**

```rust
pub struct RepoIter {
    repo: Arc<Repository>,
    tracked_iter: VecIntoIter<(BString, ObjectId)>,
    untracked_dir_stack: Vec<PathBuf>,
    untracked_current_iter: Option<ReadDir>,
    tracked_index_set: HashSet<BString>,  // Pre-computed for O(1) lookup
    work_dir: PathBuf,
    head_tree_id: ObjectId,
    mode: IterMode,
    filter: Option<Vec<StatusChar>>,
}

pub enum IterMode {
    Tracked,
    Untracked,
    Both,
}

pub enum RepoItem {
    Tracked(FileStatus),
    Untracked(FileStatus),
}
```

**Builder pattern:**
```rust
RepoIter::builder(Arc::clone(&repo))
    .mode(IterMode::Both)
    .filter(vec![StatusChar::Added, StatusChar::Modified])
    .build()?
```

**Advantages:**
- Single pass through filesystem (for untracked files)
- Pre-computed index HashSet for O(1) lookups
- Can optionally compute FileStatus + metadata in one pass
- Consumer chooses what they want via builder

**Implementation strategy:**
1. Build index entries iterator (from StatusIter)
2. Build index HashSet (from StatusIter)
3. Walk filesystem for untracked files (from UntrackedIter, using HashSet)
4. Yield either tracked or untracked items based on mode
5. Filter based on status chars if provided

**Optimizations possible:**
- Compute metadata (DateIter stuff) during iteration if requested
- Single filesystem read for hash + metadata (combine StatusIter + DateIter work)

---

### Option C: Hybrid: Separate Iterators but Shared State

**Keep iterators separate, but provide shared state object:**

```rust
pub struct RepoState {
    repo: Arc<Repository>,
    work_dir: PathBuf,
    head_tree_id: ObjectId,
    index_entries: Vec<(BString, ObjectId)>,
    index_set: HashSet<BString>,  // Pre-computed!
}

impl RepoState {
    pub fn from_repo(repo: Arc<Repository>) -> Result<Self> {
        // Build all state once
        // ...
    }
}

// Iterators take &RepoState or Arc<RepoState>
StatusIter::new(Arc::clone(&state))?
UntrackedIter::new(Arc::clone(&state))?  // Uses index_set!
```

**Advantages:**
- Still separate concerns
- Pre-computed shared state
- Each iterator is still focused
- Easy to add new iterators

---

### Option D: Single Iterator with Lazy Computation

**One iterator to rule them all:**

```rust
pub struct RepoIter {
    repo: Arc<Repository>,
    work_dir: PathBuf,
    index_entries: Vec<(BString, ObjectId)>,
    untracked_dirs: Vec<PathBuf>,
    // ... other state
}

impl RepoIter {
    pub fn tracked_only(&mut self) -> impl Iterator<Item = Result<FileStatus>> + '_ { ... }
    pub fn untracked_only(&mut self) -> impl Iterator<Item = Result<FileStatus>> + '_ { ... }
    pub fn all(&mut self) -> impl Iterator<Item = Result<RepoItem>> + '_ { ... }
    pub fn with_metadata(&mut self) -> impl Iterator<Item = Result<FileStatusWithDate>> + '_ { ... }
}
```

**Advantages:**
- One data source
- Lazy computation (only compute what's asked for)
- Can compute metadata optionally

**Disadvantages:**
- More complex internal state
- Harder to return iterators from methods (lifetimes)

---

## Performance Comparison

| Scenario | Current (Separate) | Merged | Hybrid | Savings |
|----------|---------------------|---------|---------|----------|
| Get all modified files | 1 pass (read each file) | 1 pass | 1 pass | None |
| Get all untracked files | 1 pass + O(n×m) index checks | 1 pass + O(n) HashSet | 1 pass + O(n) HashSet | Significant |
| Get modified + untracked | 2 passes + O(n×m) index checks | 1 pass + O(n) HashSet | 2 passes + O(n) HashSet | 50-90% |
| Get modified + untracked + metadata | 2 passes + 2n fs reads | 1 pass + n fs reads | 2 passes + 2n fs reads | 50% fs reads |

---

## Recommendation

**Go with Option B: Merged Iterator with State Filter**

**Why:**
- Single pass for common use cases (tracked + untracked)
- Pre-computed index HashSet eliminates O(n×m) bottleneck
- Builder pattern allows consumer to express intent clearly
- Can optionally compute metadata efficiently
- Still relatively simple implementation

**Implementation plan:**
1. Create `RepoIter` struct that owns all state
2. Pre-compute index entries + HashSet during construction
3. Implement directory traversal with HashSet lookup
4. Yield `FileStatus` for both tracked and untracked
5. Support filter via builder (exclude specific statuses)
6. Optionally support metadata computation in same pass
7. Keep existing iterators for backward compatibility (or deprecate)

**Migration path:**
- Keep `StatusIter` and `UntrackedIter` for now (or mark deprecated)
- Introduce `RepoIter` as new primary iterator
- Update consumers to use `RepoIter` over time
