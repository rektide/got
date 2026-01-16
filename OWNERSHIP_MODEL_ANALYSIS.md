# Ownership Model for Repository Access in gixkit Iterators

## The Question

What ownership model should gixkit iterators use for repository access? Should they borrow from the repository, own it, share it, or extract needed data upfront?

## Decision: Arc<Repository> (Shared Ownership)

**We chose Option 4: Shared Ownership using `Arc<Repository>`.**

**Rationale:**
- Consistent API across all iterators (one pattern everywhere)
- No lifetime management needed
- Cheap cloning (Arc::clone is just pointer copy + refcount bump)
- Multiple iterators can share the same repository
- Avoids extracting all data upfront (lazy iteration preserved)
- Works well for both StatusIter and UntrackedIter

## Why This Matters

The ownership choice affects:
- **API ergonomics**: How easy is it to use the iterators?
- **Lifetime complexity**: Do consumers need to manage lifetimes?
- **Performance**: Are we cloning data unnecessarily?
- **Flexibility**: Can iterators be stored, chained, or passed around?

## Current State: What Do Iterators Actually Need?

Let's analyze what each iterator needs from the repository and **when**:

### StatusIter

**Repository access:**
- Construction (`new()`):
  - `repo.index()` - get all index entries (once)
  - `repo.work_dir()` - get working directory path (once)
  - `get_head_tree(&repo)` - get HEAD tree OID (once)
- Iteration (`next()`):
  - `repo.find_tree(head_tree_id)` - get head tree object (per file, for computing status)

**Data needed per iteration:** Head tree lookup, file metadata computation

### UntrackedIter

**Repository access:**
- Construction (`new()`):
  - `repo.work_dir()` - get working directory path (once)
- Iteration (`next()`):
  - `repo.index()` - check if file is in index (per file)

**Data needed per iteration:** Index lookup to filter tracked files

### Decorators (StatusFilterIter, DateIter)

**Repository access:**
- None - these are wrappers around other iterators
- They delegate all work to the inner iterator

**Key insight:** Decorators' ownership model is **determined by the inner iterator**, not the decorator itself.

## The Core Trade-off

| Requirement | Borrow Model | Ownership Model | Data Extraction |
|------------|--------------|-----------------|-----------------|
| Construction can borrow repo | ✅ Yes | ❌ No | ✅ Yes |
| Iteration needs repo access | ✅ Yes (via borrowed ref) | ✅ Yes (via owned repo) | ❌ No (must have all data upfront) |
| Consumer must clone repo | ❌ No | ✅ Yes | ❌ No |
| Consumer manages lifetimes | ✅ Yes | ❌ No | ❌ No |
| Iterator can be stored freely | ❌ No (tied to repo lifetime) | ✅ Yes | ✅ Yes |

## Options

### Option 1: Borrow Model (Reference)

**Pattern:**
```rust
pub struct StatusIter<'repo> {
    repo: &'repo Repository,
    // ... other fields
}

impl<'repo> StatusIterBuilder<'repo> {
    pub fn new(repo: &'repo Repository) -> Self { ... }
}
```

**How it works:**
- Builder takes `&repo`
- Iterator stores `&repo` (borrows for lifetime `'repo`)
- All repo access happens through the reference

**Pros:**
- ✅ No cloning needed
- ✅ Easy to use when repo is available
- ✅ Reference semantics (no ownership surprises)

**Cons:**
- ❌ Lifetime management required
- ❌ Iterator tied to repo lifetime
- ❌ Can't easily store/return iterators
- ❌ Composing chains of iterators becomes complex

**When to use:**
- Consumer has repo available for the duration of iteration
- Simple one-off iteration patterns
- Minimal iterator composition needed

---

### Option 2: Ownership Model

**Pattern:**
```rust
pub struct StatusIter {
    repo: Repository,
    // ... other fields
}

impl StatusIterBuilder {
    pub fn new(repo: Repository) -> Self { ... }
}
```

**How it works:**
- Builder takes `repo` (owned)
- Iterator owns the repository
- Consumer must `.clone()` if they still need the repo afterward

**Pros:**
- ✅ No lifetimes needed
- ✅ Iterator can be stored/returned freely
- ✅ Easy to compose chains of iterators
- ✅ More idiomatic for libraries

**Cons:**
- ❌ Requires cloning repository if consumer still needs it
- ❌ Ownership transfer can be confusing
- ❌ Might clone more data than needed

**When to use:**
- Public library APIs
- Complex iterator composition
- Consumers need to store/return iterators
- You want to avoid lifetime complexity

---

### Option 3: Data Extraction Model

**Pattern:**
```rust
pub struct StatusIter {
    head_tree_id: ObjectId,
    index_entries: Vec<(BString, ObjectId)>,
    work_dir: PathBuf,
    // No repo reference needed!
}
```

**How it works:**
- Builder takes `&repo` (borrow during construction)
- Extract all needed data into owned structures during `build()`
- Iterator stores only owned data, no repo reference

**Pros:**
- ✅ No lifetimes needed (data is owned)
- ✅ No cloning of repository (only data needed)
- ✅ Minimal overhead (only what's actually used)
- ✅ Iterator can be stored freely

**Cons:**
- ❌ Must anticipate all data needs upfront
- ❌ Can't lazy-load data during iteration
- ❌ If new data need is discovered, API changes required
- ❌ Not suitable if iteration needs repo access (e.g., UntrackedIter's per-file index lookups)

**When to use:**
- All data needs are known upfront
- No per-file repository access needed during iteration
- Works well for: StatusIter (can extract all data upfront)

---

### Option 4: Shared Ownership Model (Arc)

**Pattern:**
```rust
pub struct StatusIter {
    repo: Arc<Repository>,
    // ... other fields
}

impl StatusIterBuilder {
    pub fn new(repo: Arc<Repository>) -> Self { ... }
}
```

**How it works:**
- Builder takes `Arc<Repository>` (reference-counted)
- Multiple iterators can share the same repository
- Reference counting adds minimal overhead

**Pros:**
- ✅ No lifetimes needed
- ✅ Can share repository across multiple iterators
- ✅ No cloning (just Arc::clone, which is cheap)
- ✅ Flexible for complex compositions

**Cons:**
- ❌ Adds reference counting overhead
- ❌ Slightly more complex to use
- ❌ Overkill for simple use cases
- ❌ Still tied to the Arc's lifetime (though longer than a borrow)

**When to use:**
- Multiple iterators need the same repository
- Complex compositions with shared state
- Want flexibility without cloning overhead

---

### Option 5: Hybrid Approach

**Pattern:**
```rust
// For iterators that can extract all data upfront:
pub struct StatusIter {
    // No repo reference - all data owned
    head_tree_id: ObjectId,
    index_entries: Vec<...>,
}

// For iterators that need per-file repo access:
pub struct UntrackedIter {
    repo: Repository,  // Owned
}
```

**How it works:**
- Choose the best model per iterator based on needs
- Some iterators use data extraction, some use ownership
- No one-size-fits-all rule

**Pros:**
- ✅ Optimal for each iterator's needs
- ✅ Can avoid overhead where possible
- ✅ Flexible

**Cons:**
- ❌ Inconsistent API across iterators
- ❌ Harder to learn and remember
- ❌ Composing different models can be tricky

**When to use:**
- When each iterator has distinct requirements
- You're willing to accept API inconsistency for optimal behavior

---

## Detailed Analysis by Iterator

### StatusIter

**Iteration needs:** `repo.find_tree(head_tree_id)` per file

**Can we use data extraction?** **Yes!**
- Extract all index entries upfront
- Store `head_tree_id` instead of tree reference
- For each file: call `repo.find_tree(head_tree_id)` - BUT we need the repo for this!

**Wait, this is a problem.** StatusIter needs repo access **during iteration** to compute status by comparing with head tree.

**But hold on** - can we extract the head tree data too?
- We could store the entire tree as owned data... but that would be huge
- Or we could compute all statuses upfront during construction

**Conclusion:** StatusIter cannot use pure data extraction without computing all statuses upfront (which defeats lazy iteration).

**Best options for StatusIter:**
1. Borrow model - simple, but lifetime-bound
2. Ownership model - no lifetimes, but requires clone
3. Shared ownership (Arc) - flexible, cheap clone

---

### UntrackedIter

**Iteration needs:** `repo.index()` per file to check if tracked

**Can we use data extraction?** **Yes!**
- Extract all index paths upfront into a `HashSet<BString>`
- Then check `if tracked_set.contains(path)` during iteration (no repo needed!)

**Performance consideration:**
- If index has 10,000 entries, HashSet lookup is O(1) vs O(n) linear scan
- Memory: HashSet overhead but no repo reference

**Best options for UntrackedIter:**
1. **Data extraction** - extract all index paths into HashSet, no repo needed during iteration ✅
2. Borrow model - simple but lifetime-bound
3. Ownership model - no lifetimes, but requires clone

---

### Decorators (StatusFilterIter, DateIter)

**Repository needs:** None - they delegate to inner iterator

**Ownership model:** Determined by inner iterator's choice
- If inner borrows repo → decorator inherits lifetimes
- If inner owns repo → decorator owns it too
- If inner uses data extraction → decorator works with owned data

**Conclusion:** Decorators are agnostic - they follow whatever the inner iterator uses.

---

## Recommended Approach

### Chosen Approach: Option 4 (Arc / Shared Ownership)

**Use `Arc<Repository>` for all iterators.**

1. **StatusIter**: Uses `Arc<Repository>`
   - Needs repo access during iteration for tree lookups
   - Arc provides shared access without cloning the actual repository

2. **UntrackedIter**: Uses `Arc<Repository>`
   - Checks tracked status per-file using repo.index()
   - Arc allows sharing without upfront HashSet extraction

3. **Decorators**: Follow inner iterator's model
   - Inherit the Arc-based pattern from what they wrap

### Why Arc (Shared Ownership)?

**Consistency:**
- Same pattern everywhere in the API
- Easy to learn and remember
- No model-specific decisions per iterator

**Flexibility:**
- Multiple iterators can share the same repository
- Easy to create iterators on demand
- No lifetime complexity

**Performance:**
- Arc::clone is very cheap (pointer copy + atomic increment)
- No need to extract all data upfront
- Lazy iteration preserved

**Ergonomics:**
```rust
let repo = Arc::new(open_repo()?);
let iter1 = StatusIter::builder(Arc::clone(&repo)).build()?;
let iter2 = UntrackedIter::builder(Arc::clone(&repo)).build()?;
```

### Alternative: Option 4 (Shared Ownership / Arc)

**If you want consistency across all iterators:**
- Use `Arc<Repository>` everywhere
- No lifetimes
- Cheap cloning (Arc::clone is just pointer copy + refcount bump)
- Multiple iterators can share the same repo

**When to choose this over hybrid:**
- You value API consistency over optimization
- Multiple iterators will share the same repository
- Simpler mental model (one pattern everywhere)

**This is what we chose!** ✅

## Implementation Guide

### For StatusIter (Ownership Model)

```rust
pub struct StatusIter {
    repo: Repository,
    head_tree_id: gix_hash::ObjectId,  // Extracted OID, not tree reference
    work_dir: PathBuf,
    index_entries: std::vec::IntoIter<(BString, ObjectId)>,
}

impl StatusIter {
    fn compute_index_status(&self, path: BString, entry_oid: ObjectId) -> (char, char) {
        // Get tree from repo using stored ID
        let head_tree = self.repo.find_tree(self.head_tree_id).ok();
        // ... compute status
    }
}
```

### For UntrackedIter (Data Extraction)

```rust
pub struct UntrackedIter {
    tracked_files: std::collections::HashSet<BString>,  // No repo needed!
    work_dir: PathBuf,
    dir_stack: Vec<PathBuf>,
    current_dir_iter: Option<std::fs::ReadDir>,
    filter: UntrackedFilter,
}

impl UntrackedIter {
    fn new(repo: &Repository, ...) -> Result<Self> {
        // Extract all tracked paths during construction
        let index = repo.index()?;
        let tracked_files: HashSet<BString> = index.entries()
            .map(|e| e.path(&index).to_owned())
            .collect();

        Ok(Self {
            tracked_files,  // Owned, no repo reference needed
            // ...
        })
    }

    fn path_is_tracked(&self, path: &BStr) -> bool {
        self.tracked_files.contains(path)  // Fast O(1) lookup
    }
}
```

## Decision Matrix

| Question | Answer | Our Choice |
|----------|--------|-------------|
| Does iterator need repo during iteration? | Yes | Arc<Repository> ✅ |
| Can all needed data be extracted upfront? | Partially | Arc (preserves lazy iteration) |
| Is simple one-off iteration common? | Yes | Arc (simple `Arc::clone`) |
| Do consumers need to store/return iterators? | Yes | Arc (no lifetimes) |
| Is consistency across iterators important? | Yes | Arc everywhere ✅ |
| Is performance critical for large repos? | Yes | Arc (cheap clone, lazy iteration) |

## Implementation Details

### What Was Changed

**StatusIter:**
- Changed `repo: Repository` → `repo: Arc<Repository>`
- Changed `Builder::new(repo: Repository)` → `Builder::new(repo: Arc<Repository>)`
- Callers now use: `StatusIter::builder(Arc::clone(&repo))`

**UntrackedIter:**
- Changed `repo: Repository` → `repo: Arc<Repository>`
- Changed `Builder::new(repo: Repository)` → `Builder::new(repo: Arc<Repository>)`
- Callers now use: `UntrackedIter::builder(Arc::clone(&repo))`
- Kept per-file index lookup during iteration (no upfront HashSet extraction)

**Consumer pattern:**
```rust
let repo = open_repo(".")?;
let repo = Arc::new(repo);  // Wrap once

// Can clone Arc cheaply for multiple iterators
let iter1 = StatusIter::builder(Arc::clone(&repo)).build()?;
let iter2 = UntrackedIter::builder(Arc::clone(&repo)).build()?;
```

### Technical Notes

- Arc::clone is just an atomic increment + pointer copy (very cheap)
- Reference counting has minimal overhead
- Repository object is shared but immutable during iteration
- All iterators can safely access the same repository

## Summary

**The right model depends on the iterator's actual needs:**

1. **Extract all data you can during construction** (Option 3)
   - Best for iterators that don't need repo during iteration
   - No lifetimes, no cloning, optimal performance
   - Works for: UntrackedIter

2. **Own the repo when you must use it during iteration** (Option 2)
   - Best for iterators that need repo access per iteration
   - No lifetimes, flexible ownership
   - Works for: StatusIter

3. **Use Arc when you want shared ownership with consistency** (Option 4)
   - Best for complex compositions with shared state
   - No lifetimes, cheap clone, consistent API
   - Works for: All iterators

4. **Avoid borrow model unless you have a specific reason** (Option 1)
   - Only when consumers always have repo available during iteration
   - Acceptable for internal/private APIs or simple scripts

**The hybrid approach (1 + 2) gives the best of both worlds:**
- Optimal for each iterator's needs
- Most iterators end up with no repo reference
- When repo is needed, ownership provides flexibility
