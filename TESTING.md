# Testing Documentation

## Unit Tests Overview

Clip-Vault menggunakan Rust's built-in testing framework untuk unit tests.

### Test Coverage

#### 1. **Models Tests** (`src/models.rs`)
- ✅ ContentType serialization (to_string/from_string)
- ✅ ClipboardEntry creation (text & image)
- ✅ Preview generation dari entries
- ✅ Preview truncation (200 chars max)
- ✅ Metadata preservation

**Total: 8 tests**

#### 2. **Storage/Database Tests** (`src/storage/mod.rs`)
- ✅ Schema initialization
- ✅ Save & retrieve entries (text & image)
- ✅ History pagination & ordering
- ✅ Delete operations
- ✅ Pin/unpin functionality
- ✅ Pinned entries priority
- ✅ Search functionality
- ✅ Case-insensitive search
- ✅ Clear history (with/without pinned)
- ✅ Pagination

**Total: 14 tests**

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_save_and_retrieve_text_entry

# Run tests for specific module
cargo test models::
cargo test storage::
```

### Test Dependencies

```toml
[dev-dependencies]
tempfile = "3.8"  # For temporary test databases
```

### Adding New Tests

1. Add test function with `#[test]` attribute
2. Place in `#[cfg(test)] mod tests` block
3. Use descriptive names: `test_<functionality>_<scenario>`
4. Include assertions with clear error messages

Example:
```rust
#[test]
fn test_save_entry_returns_valid_id() {
    let (store, _temp) = create_test_store();
    let entry = ClipboardEntry::new_text("Test".to_string(), None);
    
    let id = store.save_entry(&entry).unwrap();
    
    assert!(id > 0, "Expected positive ID, got {}", id);
}
```
