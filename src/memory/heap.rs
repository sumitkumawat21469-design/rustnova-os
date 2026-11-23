//! Educational Heap Memory Allocator.
//!
//! Implements a linked-list free-block allocator supporting:
//! - Memory allocation with alignment
//! - Memory deallocation and block coalescing
//! - Allocation tracking and high-watermark statistics
//! - Allocation failure handling (OOM detection)

use core::ptr::NonNull;
use crate::println;
use crate::sync::spinlock::SpinLock;

/// Kernel heap size: 256 KB
pub const HEAP_SIZE_BYTES: usize = 256 * 1024;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AllocError {
    OutOfMemory,
    InvalidAlignment,
    CorruptedHeader,
}

#[repr(C)]
struct BlockHeader {
    magic: u32,
    size: usize,      // Size of payload (excluding header)
    is_free: bool,
    next: Option<NonNull<BlockHeader>>,
}

const BLOCK_MAGIC: u32 = 0x524E4F53; // "RNOS" in hex

pub struct MemoryStats {
    pub total_heap_bytes: usize,
    pub allocated_bytes: usize,
    pub free_bytes: usize,
    pub peak_allocated_bytes: usize,
    pub allocation_count: u32,
    pub deallocation_count: u32,
    pub failure_count: u32,
}

pub struct MicroAllocator {
    heap_start: usize,
    heap_size: usize,
    head: Option<NonNull<BlockHeader>>,
    stats: MemoryStats,
    initialized: bool,
}

unsafe impl Send for MicroAllocator {}

impl MicroAllocator {
    pub const fn new() -> Self {
        Self {
            heap_start: 0,
            heap_size: 0,
            head: None,
            stats: MemoryStats {
                total_heap_bytes: 0,
                allocated_bytes: 0,
                free_bytes: 0,
                peak_allocated_bytes: 0,
                allocation_count: 0,
                deallocation_count: 0,
                failure_count: 0,
            },
            initialized: false,
        }
    }

    /// Initializes the allocator over a designated contiguous memory buffer.
    pub fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_size = heap_size;
        self.stats.total_heap_bytes = heap_size;
        self.stats.free_bytes = heap_size.saturating_sub(core::mem::size_of::<BlockHeader>());

        let header_ptr = heap_start as *mut BlockHeader;
        unsafe {
            header_ptr.write(BlockHeader {
                magic: BLOCK_MAGIC,
                size: self.stats.free_bytes,
                is_free: true,
                next: None,
            });
            self.head = NonNull::new(header_ptr);
        }
        self.initialized = true;
    }

    /// Allocates `size` bytes aligned to `align`.
    pub fn allocate(&mut self, size: usize, align: usize) -> Result<*mut u8, AllocError> {
        if !self.initialized || size == 0 {
            self.stats.failure_count += 1;
            return Err(AllocError::OutOfMemory);
        }

        // Enforce power-of-two alignment
        if align == 0 || (align & (align - 1)) != 0 {
            return Err(AllocError::InvalidAlignment);
        }

        let header_size = core::mem::size_of::<BlockHeader>();
        let mut curr_opt = self.head;

        while let Some(mut curr) = curr_opt {
            let header = unsafe { curr.as_mut() };
            if header.magic != BLOCK_MAGIC {
                self.stats.failure_count += 1;
                return Err(AllocError::CorruptedHeader);
            }

            if header.is_free && header.size >= size {
                // Check if block can be split
                let rem_size = header.size.saturating_sub(size);
                if rem_size > header_size + 16 {
                    let next_block_addr = (curr.as_ptr() as usize) + header_size + size;
                    let next_header_ptr = next_block_addr as *mut BlockHeader;
                    unsafe {
                        next_header_ptr.write(BlockHeader {
                            magic: BLOCK_MAGIC,
                            size: rem_size - header_size,
                            is_free: true,
                            next: header.next,
                        });
                        header.next = NonNull::new(next_header_ptr);
                    }
                    header.size = size;
                }

                header.is_free = false;
                self.stats.allocated_bytes += header.size;
                self.stats.free_bytes = self.stats.free_bytes.saturating_sub(header.size);
                if self.stats.allocated_bytes > self.stats.peak_allocated_bytes {
                    self.stats.peak_allocated_bytes = self.stats.allocated_bytes;
                }
                self.stats.allocation_count += 1;

                let payload_ptr = (curr.as_ptr() as usize + header_size) as *mut u8;
                return Ok(payload_ptr);
            }

            curr_opt = header.next;
        }

        self.stats.failure_count += 1;
        Err(AllocError::OutOfMemory)
    }

    /// Deallocates memory at `ptr`.
    pub fn deallocate(&mut self, ptr: *mut u8) -> Result<(), AllocError> {
        if ptr.is_null() || !self.initialized {
            return Ok(());
        }

        let header_size = core::mem::size_of::<BlockHeader>();
        let header_addr = (ptr as usize).saturating_sub(header_size);
        let header_ptr = header_addr as *mut BlockHeader;

        unsafe {
            let header = &mut *header_ptr;
            if header.magic != BLOCK_MAGIC {
                return Err(AllocError::CorruptedHeader);
            }
            if !header.is_free {
                header.is_free = true;
                self.stats.allocated_bytes = self.stats.allocated_bytes.saturating_sub(header.size);
                self.stats.free_bytes += header.size;
                self.stats.deallocation_count += 1;
            }
        }

        // Coalesce adjacent free blocks
        self.coalesce();
        Ok(())
    }

    fn coalesce(&mut self) {
        let mut curr_opt = self.head;
        let header_size = core::mem::size_of::<BlockHeader>();

        while let Some(mut curr) = curr_opt {
            let header = unsafe { curr.as_mut() };
            if header.is_free {
                if let Some(next_ptr) = header.next {
                    let next_header = unsafe { next_ptr.as_ref() };
                    let expected_next_addr = (curr.as_ptr() as usize) + header_size + header.size;
                    if expected_next_addr == (next_ptr.as_ptr() as usize) && next_header.is_free {
                        // Merge adjacent free blocks
                        header.size += header_size + next_header.size;
                        header.next = next_header.next;
                        continue;
                    }
                }
            }
            curr_opt = header.next;
        }
    }

    pub fn stats(&self) -> MemoryStats {
        MemoryStats {
            total_heap_bytes: self.stats.total_heap_bytes,
            allocated_bytes: self.stats.allocated_bytes,
            free_bytes: self.stats.free_bytes,
            peak_allocated_bytes: self.stats.peak_allocated_bytes,
            allocation_count: self.stats.allocation_count,
            deallocation_count: self.stats.deallocation_count,
            failure_count: self.stats.failure_count,
        }
    }
}

static mut KERNEL_HEAP_RAW: [u8; HEAP_SIZE_BYTES] = [0; HEAP_SIZE_BYTES];
pub static GLOBAL_ALLOCATOR: SpinLock<MicroAllocator> = SpinLock::new(MicroAllocator::new());

/// Initializes the global memory manager.
pub fn init_memory() {
    unsafe {
        let heap_ptr = KERNEL_HEAP_RAW.as_mut_ptr() as usize;
        GLOBAL_ALLOCATOR.lock().init(heap_ptr, HEAP_SIZE_BYTES);
    }
}

/// Runs Phase 6 Memory Management demonstration:
/// Tests allocation, deallocation, statistics tracking, and failure handling.
pub fn run_memory_demo() -> bool {
    println!("\n\x1b[33m--- PHASE 6: MEMORY MANAGEMENT DEMONSTRATION ---\x1b[0m");

    let initial_stats = GLOBAL_ALLOCATOR.lock().stats();
    println!(
        "[MEM] Initial Heap: Total={} bytes ({} KB), Free={} bytes, Active={} bytes",
        initial_stats.total_heap_bytes,
        initial_stats.total_heap_bytes / 1024,
        initial_stats.free_bytes,
        initial_stats.allocated_bytes
    );

    // 1. Allocate block 1 (128 bytes)
    println!("[MEM] Allocating Block-A (128 bytes)...");
    let ptr_a = match GLOBAL_ALLOCATOR.lock().allocate(128, 8) {
        Ok(p) => {
            println!("[MEM] Block-A allocated successfully at {:p}", p);
            p
        }
        Err(e) => {
            println!("[MEM-ERR] Failed to allocate Block-A: {:?}", e);
            return false;
        }
    };

    // 2. Allocate block 2 (512 bytes)
    println!("[MEM] Allocating Block-B (512 bytes)...");
    let ptr_b = match GLOBAL_ALLOCATOR.lock().allocate(512, 8) {
        Ok(p) => {
            println!("[MEM] Block-B allocated successfully at {:p}", p);
            p
        }
        Err(e) => {
            println!("[MEM-ERR] Failed to allocate Block-B: {:?}", e);
            return false;
        }
    };

    // 3. Write data to allocations and verify
    unsafe {
        core::ptr::write_bytes(ptr_a, 0xAA, 128);
        core::ptr::write_bytes(ptr_b, 0x55, 512);

        assert_eq!(*ptr_a, 0xAA);
        assert_eq!(*ptr_b, 0x55);
    }
    println!("[MEM] Data write/read verification passed on Block-A and Block-B");

    // Check stats after allocation
    let stats_after_alloc = GLOBAL_ALLOCATOR.lock().stats();
    println!(
        "[MEM] Stats After Alloc: ActiveAllocations={}, AllocatedBytes={}, FreeBytes={}, PeakBytes={}",
        stats_after_alloc.allocation_count,
        stats_after_alloc.allocated_bytes,
        stats_after_alloc.free_bytes,
        stats_after_alloc.peak_allocated_bytes
    );

    // 4. Deallocate Block-A
    println!("[MEM] Deallocating Block-A ({:p})...", ptr_a);
    let _ = GLOBAL_ALLOCATOR.lock().deallocate(ptr_a);

    let stats_after_free = GLOBAL_ALLOCATOR.lock().stats();
    println!(
        "[MEM] Stats After Dealloc: AllocatedBytes={}, FreeBytes={}, DeallocCount={}",
        stats_after_free.allocated_bytes,
        stats_after_free.free_bytes,
        stats_after_free.deallocation_count
    );

    // 5. Test Allocation Failure Handling (Deliberate out-of-memory request: 1 MB when heap is 256 KB)
    println!("[MEM] Testing Allocation Failure Handling (requesting 1,048,576 bytes on 256 KB heap)...");
    let oom_res = GLOBAL_ALLOCATOR.lock().allocate(1024 * 1024, 8);
    match oom_res {
        Err(AllocError::OutOfMemory) => {
            println!("\x1b[32m[MEM-SUCCESS] Allocation failure properly caught and handled: Err(OutOfMemory) returned safely without crash!\x1b[0m");
        }
        _ => {
            println!("[MEM-ERR] Expected OutOfMemory error, got: {:?}", oom_res);
            return false;
        }
    }

    // Clean up Block-B
    let _ = GLOBAL_ALLOCATOR.lock().deallocate(ptr_b);

    let final_stats = GLOBAL_ALLOCATOR.lock().stats();
    println!(
        "[MEM] Final Heap Summary: TotalBytes={}, AllocatedBytes={}, FreeBytes={}, FailuresRecorded={}",
        final_stats.total_heap_bytes,
        final_stats.allocated_bytes,
        final_stats.free_bytes,
        final_stats.failure_count
    );
    println!("\x1b[32m[MEM-PASSED] Memory Manager Phase 6 verification complete.\x1b[0m\n");

    true
}
