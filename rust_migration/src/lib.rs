//! Safe Rust implementation of dlib's memory manager
//!
//! This crate provides a safe, thread-safe memory management implementation
//! that replaces the original C++ implementation while maintaining similar
//! performance characteristics through memory pooling.

mod memory_manager;
pub use memory_manager::MemoryManager;
pub use memory_manager::MemoryBlock;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_allocation() {
        let manager = MemoryManager::<u32>::new(5);
        let block = manager.allocate();
        assert_eq!(manager.get_number_of_allocations(), 1);
        drop(block);
        assert_eq!(manager.get_number_of_allocations(), 0);
    }
}
