# DLib Memory Manager - Rust Implementation

This is a safe Rust implementation of DLib's memory manager, addressing several memory safety concerns present in the original C++ implementation.

## Key Improvements

### Memory Safety
- Replaced manual memory management with Rust's ownership system
- Eliminated use-after-free vulnerabilities through RAII
- Removed unsafe type casting with proper generic implementations
- Added thread-safety guarantees using `Arc` and `Mutex`

### Original Issues Addressed
1. **Use-after-free Prevention**
   - Original: Manual pool management could lead to use-after-free
   - Fixed: Memory blocks are managed through RAII wrapper

2. **Thread Safety**
   - Original: No thread-safety guarantees
   - Fixed: Thread-safe pool management with `parking_lot::Mutex`

3. **Type Safety**
   - Original: Unsafe union and reinterpret casts
   - Fixed: Safe generic implementations

### Usage Example

```rust
use dlib_memory_manager::MemoryManager;

// Create a memory manager with pool size of 10
let manager = MemoryManager::<u32>::new(10);

// Allocate a value
let mut block = manager.allocate();
*block = 42;

// Memory automatically returns to pool when block is dropped
drop(block);
```

## Implementation Details

The implementation maintains the performance benefits of pooling while providing:
- RAII guarantees
- Thread safety
- Type safety
- Exception safety
- Memory leak prevention

## Original vs New Implementation

### Original C++ (Unsafe)
```cpp
T* temp = reinterpret_cast<T*>(next);
// Manual memory management
item->~T();
::operator delete(static_cast<void*>(item));
```

### New Rust (Safe)
```rust
// Safe memory management through RAII
pub struct MemoryBlock<'a, T> {
    item: Option<Box<T>>,
    manager: &'a MemoryManager<T>,
}
```

## Testing
Run tests with:
```bash
cargo test
```
