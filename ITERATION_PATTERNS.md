# Iteration Patterns in gixkit

## Current Iteration Patterns

| Iterator | File | Pattern | Ownership | Idiomatic? | Notes |
|----------|------|---------|------------|--------------|--------|
| `StatusIter` | status.rs | `while let Some((path, oid)) = self.index_entries.next()` | ✅ `into_iter()` | ✅ Yes | Uses consumer-owned Vec iterator |
| `UntrackedIter` | untracked.rs | `while { loop { if is_none() { pop() } if let Some(iter) { process() } }` | ❌ Borrow | ⚠️ Partial | Directory traversal with dir_stack; borrows repo |
| `StatusFilterIter` | filters.rs | `loop { match self.inner.next()? }` | ❌ Borrow | ⚠️ Partial | Delegates to inner, borrows |
| `DateIter` | decorators.rs | `match self.inner.next()? { Ok(status) => {...} Err(e) => ... }` | ❌ Borrow | ✅ Yes | Wrapper decorator pattern |
| `get_index_entries` | index.rs | `for entry in index.entries()` | ❌ Borrow | ✅ Yes | Helper function, returns owned Vec |
| `path_in_index` | index.rs | `for (p, _) in index.entries_with_paths_by_filter_map(...)` | ❌ Borrow | ✅ Yes | Early break with `found` flag |

## Issues Identified

### 1. Inconsistent Ownership Model

Some iterators transfer ownership to consumers:
- `StatusIter`: Uses `std::vec::IntoIter` ✅
- `get_index_entries`: Returns `Vec<IndexedFile>` ✅
- `path_in_index`: Uses idiomatic iteration ✅

Others borrow from repository (lifetime-bound):
- `UntrackedIter`: Stores `&'repo Repository` ❌
- `StatusFilterIter`: Borrows from inner iterator ❌
- `DateIter`: Borrows from inner iterator ❌

**Impact**: Makes API inconsistent. Consumers can't easily store/chain iterators.

### 2. UntrackedIter Borrows Repo

```rust
pub struct UntrackedIter<'repo> {
    repo: &'repo Repository,  // ❌ Borrow
    work_dir: PathBuf,
    dir_stack: Vec<PathBuf>,
    current_dir_iter: Option<std::fs::ReadDir>,
    filter: crate::types::UntrackedFilter,
}
```

**Issue**: Consumer can't own the iterator because it's tied to repo lifetime.

### 3. StatusFilterIter Borrows from Inner

```rust
pub struct StatusFilterIter<I> {
    inner: I,  // ❌ Borrowed
    filter: StatusFilter,
}
```

**Issue**: Pattern is fine for filtering, but when wrapping borrowed iterators, the borrow propagates.

### 4. Manual Directory Traversal Pattern

Both `StatusIter` and `UntrackedIter` use manual `dir_stack` + `current_dir_iter` for tree walking.

**Pros**:
- Memory efficient (no recursion)
- Controlled depth

**Cons**:
- More complex than using `walkdir` crate
- Manual state management

## Recommendations

### Option 1: Consistent Borrow Model (Easiest) ⭐ Recommended

**Keep everything borrowing.** All iterators use `&'repo Repository` lifetimes.

**Pros**:
- Minimal changes required
- No ownership transfer complexity
- Consistent with current `UntrackedIter`, `StatusFilterIter`, `DateIter`

**Cons**:
- Lifetime management burden on consumer
- Can't easily store iterators in structs/collections

**When to use**: Small scripts, direct consumption patterns, minimal composition needs.

### Option 2: Consistent Ownership Model (Most Flexible)

**Make everything ownership-based.** Each iterator owns its data.

**Implementation**: Use `.into_iter()` everywhere for collection-to-iterator conversion.

**Pros**:
- Consumers can store iterators freely
- More idiomatic Rust (avoid excessive lifetimes)
- Better for advanced use cases (chaining, collecting, etc.)

**Cons**:
- Requires cloning repository state if needed
- More complex to implement for some patterns

**When to use**: Libraries, public APIs, complex iterator composition, when consumers need flexibility.

### Option 3: Stream-based/Generator Model (Advanced)

Use async streams or generators for large repositories.

**Pros**:
- Lazy evaluation without storing all state
- Memory efficient
- Cancels easily

**Cons**:
- Complex to implement
- Adds async complexity

**When to use**: Large repositories, memory-constrained environments, progressive output needs.

## Chosen Recommendation: Option 2

**Go with consistent ownership model using `into_iter()` everywhere.**

**Rationale**:
- You're already doing this successfully with `StatusIter`
- Makes library easier to use (no lifetime management burden on consumer)
- More idiomatic Rust
- Consumer can `.collect()` into any collection they want
- Fits your stated goal: "consumer of the iterator to end up with ownership as they iterate"

**Implementation notes**:
1. Use `collection.into_iter()` for owned iteration
2. Clone minimal required data (paths, OIDs)
3. Avoid storing `&Repository` in iterators if possible
4. For operations requiring repo access, either:
   - Pass `&repo` as method argument
   - Or clone minimal needed data up front
