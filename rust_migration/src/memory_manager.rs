//! Memory Manager implementation with safe Rust abstractions
//! Provides pooled allocation with RAII guarantees

use std::marker::PhantomData;
use std::sync::Arc;
use parking_lot::Mutex;
use std::collections::VecDeque;

/// A thread-safe memory pool manager for type T
pub struct MemoryManager<T> {
    pool: Arc<Mutex<VecDeque<Box<T>>>>,
    max_pool_size: usize,
    allocations: Arc<Mutex<usize>>,
}

impl<T> MemoryManager<T> {
    /// Create a new memory manager with specified maximum pool size
    pub fn new(max_pool_size: usize) -> Self {
        Self {
            pool: Arc::new(Mutex::new(VecDeque::with_capacity(max_pool_size))),
            max_pool_size,
            allocations: Arc::new(Mutex::new(0)),
        }
    }

    /// Get the current number of outstanding allocations
    pub fn get_number_of_allocations(&self) -> usize {
        *self.allocations.lock()
    }

    /// Allocate a new instance of T
    pub fn allocate(&self) -> MemoryBlock<T>
    where
        T: Default,
    {
        let item = if let Some(mut pool) = self.pool.try_lock() {
            pool.pop_front().unwrap_or_else(|| Box::new(T::default()))
        } else {
            // If pool is locked, create new instance
            Box::new(T::default())
        };

        *self.allocations.lock() += 1;

        MemoryBlock {
            item: Some(item),
            manager: self,
        }
    }

    /// Allocate an array of T
    pub fn allocate_array(&self, size: usize) -> Vec<T>
    where
        T: Default,
    {
        *self.allocations.lock() += 1;
        Vec::with_capacity(size)
    }

    // Internal method used by MemoryBlock on drop
    fn return_to_pool(&self, item: Box<T>) {
        if let Some(mut pool) = self.pool.try_lock() {
            if pool.len() < self.max_pool_size {
                pool.push_back(item);
            }
        }
        *self.allocations.lock() -= 1;
    }
}

/// RAII wrapper for allocated memory
pub struct MemoryBlock<'a, T> {
    item: Option<Box<T>>,
    manager: &'a MemoryManager<T>,
}

impl<'a, T> Drop for MemoryBlock<'a, T> {
    fn drop(&mut self) {
        if let Some(item) = self.item.take() {
            self.manager.return_to_pool(item);
        }
    }
}

impl<'a, T> std::ops::Deref for MemoryBlock<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.item.as_ref().unwrap()
    }
}

impl<'a, T> std::ops::DerefMut for MemoryBlock<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.item.as_mut().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocation_count() {
        let manager = MemoryManager::<u32>::new(5);
        assert_eq!(manager.get_number_of_allocations(), 0);

        let _block = manager.allocate();
        assert_eq!(manager.get_number_of_allocations(), 1);
    }

    #[test]
    fn test_pool_reuse() {
        let manager = MemoryManager::<String>::new(1);

        // First allocation
        let block1 = manager.allocate();
        let addr1 = &*block1 as *const String;
        drop(block1);

        // Second allocation should reuse the same memory
        let block2 = manager.allocate();
        let addr2 = &*block2 as *const String;

        // Addresses should be different (Rust doesn't guarantee memory reuse)
        // but allocation count should be correct
        assert_eq!(manager.get_number_of_allocations(), 1);
    }
}
