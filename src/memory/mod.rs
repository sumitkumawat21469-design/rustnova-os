//! Memory Management Subsystem.

pub mod frame;
pub mod heap;

pub use frame::FrameAllocator;
pub use heap::{
    init_memory, run_memory_demo, AllocError, MemoryStats, MicroAllocator, GLOBAL_ALLOCATOR,
    HEAP_SIZE_BYTES,
};
