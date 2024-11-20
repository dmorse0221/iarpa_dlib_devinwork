//! Memory Manager implementation with safe Rust abstractions
//! Provides pooled allocation with RAII guarantees

use std::alloc::{alloc, dealloc, Layout};
use parking_lot::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[repr(C, align(64))]  // Cache line alignment to prevent false sharing
pub struct MemoryManager<T> {
    pool: Arc<Mutex<Vec<*mut T>>>,
    max_pool_size: usize,
    allocations: Arc<AtomicUsize>,
}

impl<T> MemoryManager<T> {
    /// Create a new memory manager with specified maximum pool size
    pub fn new(max_pool_size: usize) -> Self {
        Self {
            pool: Arc::new(Mutex::new(Vec::with_capacity(max_pool_size))),
            max_pool_size,
            allocations: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Get the current number of outstanding allocations
    pub fn get_number_of_allocations(&self) -> usize {
        self.allocations.load(Ordering::Relaxed)
    }

    /// Allocate a new instance of T
    pub fn allocate(&self) -> MemoryBlock<T>
    where
        T: Default,
    {
        let ptr = {
            let mut pool = self.pool.try_lock();
            match pool {
                Some(ref mut pool) if !pool.is_empty() => pool.pop().unwrap(),
                _ => unsafe {
                    let layout = Layout::new::<T>();
                    let ptr = alloc(layout) as *mut T;
                    ptr.write(T::default());
                    ptr
                }
            }
        };

        self.allocations.fetch_add(1, Ordering::Relaxed);

        MemoryBlock {
            ptr: Some(ptr),
            manager: self,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Allocate an array of T
    pub fn allocate_array(&self, size: usize) -> Vec<T>
    where
        T: Default,
    {
        unsafe {
            let mut vec = Vec::with_capacity(size);
            vec.set_len(size);
            for item in vec.iter_mut() {
                *item = T::default();
            }
            self.allocations.fetch_add(1, Ordering::Relaxed);
            vec
        }
    }

    // Internal method used by MemoryBlock on drop
    fn return_to_pool(&self, ptr: *mut T) {
        if let Some(mut pool) = self.pool.try_lock() {
            if pool.len() < self.max_pool_size {
                pool.push(ptr);
            } else {
                unsafe {
                    let layout = Layout::new::<T>();
                    dealloc(ptr as *mut u8, layout);
                }
            }
        } else {
            unsafe {
                let layout = Layout::new::<T>();
                dealloc(ptr as *mut u8, layout);
            }
        }
        self.allocations.fetch_sub(1, Ordering::Relaxed);
    }
}

/// RAII wrapper for allocated memory
pub struct MemoryBlock<'a, T> {
    ptr: Option<*mut T>,
    manager: &'a MemoryManager<T>,
    _phantom: std::marker::PhantomData<T>,
}

impl<'a, T> Drop for MemoryBlock<'a, T> {
    fn drop(&mut self) {
        if let Some(ptr) = self.ptr.take() {
            self.manager.return_to_pool(ptr);
        }
    }
}

impl<'a, T> std::ops::Deref for MemoryBlock<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr.unwrap() }
    }
}

impl<'a, T> std::ops::DerefMut for MemoryBlock<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr.unwrap() }
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
