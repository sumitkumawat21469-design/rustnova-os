//! Physical Frame Allocator.
//!
//! Manages 4096-byte physical page frames over system memory.

pub const PAGE_SIZE: usize = 4096;

pub struct FrameAllocator {
    pub base_addr: usize,
    pub total_frames: usize,
    pub allocated_frames: usize,
}

impl FrameAllocator {
    pub const fn new() -> Self {
        Self {
            base_addr: 0x1000000, // 16 MB boundary
            total_frames: 4096,   // 16 MB / 4 KB = 4096 frames
            allocated_frames: 0,
        }
    }

    pub fn allocate_frame(&mut self) -> Option<usize> {
        if self.allocated_frames < self.total_frames {
            let addr = self.base_addr + (self.allocated_frames * PAGE_SIZE);
            self.allocated_frames += 1;
            Some(addr)
        } else {
            None
        }
    }
}
