//! Fixed-Capacity Ready Queue for no_std Environment.
//!
//! Implements a deterministic FIFO ring buffer queue for task scheduling
//! without dynamic memory allocation overhead.

pub const MAX_TASKS: usize = 32;

pub struct ReadyQueue {
    buffer: [u32; MAX_TASKS],
    head: usize,
    tail: usize,
    count: usize,
}

impl ReadyQueue {
    pub const fn new() -> Self {
        Self {
            buffer: [0; MAX_TASKS],
            head: 0,
            tail: 0,
            count: 0,
        }
    }

    pub fn push(&mut self, task_id: u32) -> bool {
        if self.count >= MAX_TASKS {
            return false;
        }
        self.buffer[self.tail] = task_id;
        self.tail = (self.tail + 1) % MAX_TASKS;
        self.count += 1;
        true
    }

    pub fn pop(&mut self) -> Option<u32> {
        if self.count == 0 {
            return None;
        }
        let task_id = self.buffer[self.head];
        self.head = (self.head + 1) % MAX_TASKS;
        self.count -= 1;
        Some(task_id)
    }

    pub fn peek(&self) -> Option<u32> {
        if self.count == 0 {
            None
        } else {
            Some(self.buffer[self.head])
        }
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn clear(&mut self) {
        self.head = 0;
        self.tail = 0;
        self.count = 0;
    }
}
