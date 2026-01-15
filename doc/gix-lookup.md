# Computing OIDs from Raw Bytes in gix Crate

## Summary

Comprehensive information about how to compute OIDs (Object IDs) from raw bytes/content in the gix crate.

## Key Finding: Current Code is Incorrect

The line `gix::odb::Data::from(content).hash()` in your code does not compile in gix version 0.63. The `gix::odb` module doesn't have a `Data` type with a `hash()` method.

## Correct API for Computing OIDs

### 1. Primary Method: `gix_object::compute_hash()`

This is the standard function for computing OIDs from raw content:

```rust
use gix_object::{Kind, compute_hash};
use gix_hash::Kind as HashKind;

// Compute OID for blob content
fn hash_blob_content(content: &[u8]) -> gix_hash::ObjectId {
    compute_hash(
        HashKind::Sha1, // Hash algorithm (usually SHA1)
        Kind::Blob,    // Object type (Blob, Tree, Commit, Tag)
        content        // Raw bytes
    )
}
```

Usage example in your porcelain tool:

```rust
if full_path.exists() {
    if let Ok(content) = std::fs::read(&full_path) {
        let oid = gix_object::compute_hash(
            gix_hash::Kind::Sha1,
            gix_object::Kind::Blob,
            &content
        );
        if oid != entry_oid {
            worktree_status = 'M';
        }
    }
}
```

**Function Signature:**

```rust
pub fn compute_hash(
    hash_kind: gix_hash::Kind, // The hash algorithm (Sha1 or Sha256)
    object_kind: Kind,         // The git object type
    data: &[u8]                // The raw content bytes
) -> gix_hash::ObjectId
```

### 2. Using `gix_object::Data` for Verification

If you need to verify checksums or work with object data:

```rust
use gix_object::Data;

// Create a Data object
let data = Data::new(Kind::Blob, &content);

// Verify checksum against expected OID
if let Err(e) = data.verify_checksum(&expected_oid) {
    // Handle checksum mismatch
}

// Or decode to examine the object
if let Ok(obj_ref) = data.decode() {
    // Access object-specific data
}
```

### 3. Recommended Approach: Use Built-in Status API

For implementing `git status --porcelain`, you should use the dedicated status API instead of manually computing OIDs:

```rust
use gix::status::index_worktree;

let repo = gix::open(".").unwrap();
let index = repo.index().unwrap();

// Use the built-in index_worktree_status function
let outcome = repo.index_worktree_status(
    &index,
    std::iter::empty::<&gix::bstr::BStr>(), // pathspecs (empty for all)
    &mut your_delegate,           // VisitEntry delegate
    &mut your_blob_comparer,       // CompareBlobs implementation
    &mut your_submodule_status,    // SubmoduleStatus implementation
    &mut your_progress,
    &std::sync::atomic::AtomicBool::new(false),
    index_worktree::Options::default()
).unwrap();
```

## Available Types and Modules

### `gix_object::Kind` - Object Types

```rust
pub enum Kind {
    Tree,
    Blob,
    Commit,
    Tag,
}
```

### `gix_hash::Kind` - Hash Algorithms

```rust
pub enum Kind {
    Sha1,   // Default for most repos
    Sha256, // Available with feature flag
}
```

### `gix_hash::ObjectId` - The Result Type

An owned hash identifying git objects (typically 20-byte SHA1).

## Hashing Mechanism

The `compute_hash` function internally:

1. Creates a loose object header: `"{object_kind} {size}\0"`
2. Creates a hasher for the specified hash algorithm
3. Updates hasher with header + data
4. Returns the digest as an ObjectId

## Alternative: Using gix Repository Methods

If you have access to a repository, there are convenience methods:

```rust
let repo = gix::open(".").unwrap();

// Compute and write blob if it doesn't exist
let oid = repo.write_blob(&content).unwrap();

// Or compute without writing
let oid = gix_object::compute_hash(
    repo.object_hash(),
    gix_object::Kind::Blob,
    &content
);
```

## File Paths in Your Project

- Current implementation: `/home/rektide/src/got/crate/porcelain/src/main.rs` (line 44 - currently broken)
- Dependencies: `/home/rektide/src/got/crate/porcelain/Cargo.toml` (using gix 0.63)

## Recommended Fix for Your Code

Replace the broken line 44 in `main.rs`:

```rust
// OLD (doesn't work):
let oid = gix::odb::Data::from(content).hash();

// NEW (correct):
let oid = gix_object::compute_hash(
    gix_hash::Kind::Sha1,
    gix_object::Kind::Blob,
    &content
);
```

## Additional Issues in Current Code

Your code also has other compilation errors that need fixing:

1. `head.try_peel_to_commit()` - method doesn't exist on `gix::Head`
2. `index.state()` - state is a private field, not a method

For a proper `git status --porcelain` implementation, I'd recommend using the `gix::status` module instead of manually computing OIDs and comparing.
