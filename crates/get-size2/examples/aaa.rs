#![allow(unsafe_code)]
use get_size2::GetSize;
use std::alloc;
use std::alloc::{GlobalAlloc, Layout};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use Ordering::Relaxed;

static USED_MEMORY: AtomicUsize = AtomicUsize::new(0);
struct WrappedRustAllocator;

unsafe impl GlobalAlloc for WrappedRustAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let wrappee = alloc::System;
        let size = layout.size();

        let alloced = wrappee.alloc(layout);
        if !alloced.is_null() {
            USED_MEMORY.fetch_add(size, Relaxed);
        }

        alloced
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let wrappee = alloc::System;
        let size = layout.size();

        wrappee.dealloc(ptr, layout);
        USED_MEMORY.fetch_sub(size, Relaxed);
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let wrappee = alloc::System;
        let old_size = layout.size();

        let realloced = wrappee.realloc(ptr, layout, new_size);
        if !realloced.is_null() {
            if new_size > old_size {
                USED_MEMORY.fetch_add(new_size - old_size, Relaxed);
            } else {
                USED_MEMORY.fetch_sub(old_size - new_size, Relaxed);
            }
        }

        realloced
    }
}

#[global_allocator]
static RUST_ALLOCATOR: WrappedRustAllocator = WrappedRustAllocator;

fn main() {
    let used_before = USED_MEMORY.load(Relaxed);
    let x = Arc::new(42u64);
    let used_after = USED_MEMORY.load(Relaxed);
    // assert_eq!(used_before, used_after);
    let delta = used_after - used_before;
    assert_eq!(x.get_heap_size(), delta);
}
