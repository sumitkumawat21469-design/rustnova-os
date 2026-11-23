//! Task Control Block (TCB) and Task Management Subsystem.
//!
//! Represents system tasks, life-cycle states (READY, RUNNING, BLOCKED, TERMINATED),
//! priorities, execution statistics, and process accounting.

use core::fmt;

/// Task Life-Cycle States required by OS CA-6 specification.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum TaskState {
    Ready,
    Running,
    Blocked,
    Terminated,
}

impl TaskState {
    pub const fn as_str(&self) -> &'static str {
        match self {
            TaskState::Ready => "READY",
            TaskState::Running => "RUNNING",
            TaskState::Blocked => "BLOCKED",
            TaskState::Terminated => "TERMINATED",
        }
    }
}

impl fmt::Display for TaskState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Task Priority Levels.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(u8)]
pub enum TaskPriority {
    Idle = 0,
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

impl TaskPriority {
    pub const fn weight(&self) -> u32 {
        match self {
            TaskPriority::Idle => 1,
            TaskPriority::Low => 2,
            TaskPriority::Normal => 4,
            TaskPriority::High => 6,
            TaskPriority::Critical => 8,
        }
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            TaskPriority::Idle => "IDLE",
            TaskPriority::Low => "LOW",
            TaskPriority::Normal => "NORMAL",
            TaskPriority::High => "HIGH",
            TaskPriority::Critical => "CRITICAL",
        }
    }
}

/// Task Execution Information and Accounting Statistics.
#[derive(Copy, Clone, Debug)]
pub struct TaskExecutionInfo {
    pub arrival_time: u64,
    pub first_scheduled_time: Option<u64>,
    pub completion_time: Option<u64>,
    pub cpu_burst_total: u32,
    pub cpu_burst_remaining: u32,
    pub cpu_time_executed: u32,
    pub wait_time: u32,
    pub turnaround_time: u32,
    pub response_time: u32,
    pub context_switches: u32,
}

impl TaskExecutionInfo {
    pub const fn new(arrival_time: u64, burst_ticks: u32) -> Self {
        Self {
            arrival_time,
            first_scheduled_time: None,
            completion_time: None,
            cpu_burst_total: burst_ticks,
            cpu_burst_remaining: burst_ticks,
            cpu_time_executed: 0,
            wait_time: 0,
            turnaround_time: 0,
            response_time: 0,
            context_switches: 0,
        }
    }
}

/// Task function signature: `(task_id: u32, step: u32) -> bool`
/// Returns `true` if task should continue executing, or `false` if completed.
pub type TaskFunction = fn(u32, u32) -> bool;

/// Task Control Block (TCB).
#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    pub id: u32,
    pub name: &'static str,
    pub state: TaskState,
    pub priority: TaskPriority,
    pub execution_info: TaskExecutionInfo,
    pub task_fn: Option<TaskFunction>,
    pub assigned_quantum: u32,
    pub remaining_quantum: u32,
}

impl TaskControlBlock {
    pub const fn new(
        id: u32,
        name: &'static str,
        priority: TaskPriority,
        burst_ticks: u32,
        arrival_time: u64,
        task_fn: Option<TaskFunction>,
    ) -> Self {
        Self {
            id,
            name,
            state: TaskState::Ready,
            priority,
            execution_info: TaskExecutionInfo::new(arrival_time, burst_ticks),
            task_fn,
            assigned_quantum: 2,
            remaining_quantum: 2,
        }
    }

    /// Marks the task as running, recording response time on first dispatch.
    pub fn mark_running(&mut self, current_tick: u64) {
        self.state = TaskState::Running;
        self.execution_info.context_switches += 1;
        if self.execution_info.first_scheduled_time.is_none() {
            self.execution_info.first_scheduled_time = Some(current_tick);
            self.execution_info.response_time = (current_tick.saturating_sub(self.execution_info.arrival_time)) as u32;
        }
    }

    /// Marks the task as ready (yielded or preempted).
    pub fn mark_ready(&mut self) {
        if self.state != TaskState::Terminated {
            self.state = TaskState::Ready;
        }
    }

    /// Marks the task as blocked on I/O or lock.
    pub fn mark_blocked(&mut self) {
        if self.state != TaskState::Terminated {
            self.state = TaskState::Blocked;
        }
    }

    /// Marks the task as terminated upon completion.
    pub fn mark_terminated(&mut self, current_tick: u64) {
        self.state = TaskState::Terminated;
        self.execution_info.completion_time = Some(current_tick);
        self.execution_info.turnaround_time = (current_tick.saturating_sub(self.execution_info.arrival_time)) as u32;
        self.execution_info.cpu_burst_remaining = 0;
    }

    /// Advances task execution by one CPU tick.
    /// Returns `true` if task is completed.
    pub fn tick_execute(&mut self, current_tick: u64) -> bool {
        self.execution_info.cpu_time_executed += 1;
        if self.execution_info.cpu_burst_remaining > 0 {
            self.execution_info.cpu_burst_remaining -= 1;
        }

        // Call task workload payload if provided
        if let Some(task_fn) = self.task_fn {
            let should_continue = task_fn(self.id, self.execution_info.cpu_time_executed);
            if !should_continue || self.execution_info.cpu_burst_remaining == 0 {
                self.mark_terminated(current_tick);
                return true;
            }
        } else if self.execution_info.cpu_burst_remaining == 0 {
            self.mark_terminated(current_tick);
            return true;
        }

        false
    }

    /// Increases waiting time for tasks waiting in the ready queue.
    pub fn increment_wait_time(&mut self) {
        if self.state == TaskState::Ready {
            self.execution_info.wait_time += 1;
        }
    }
}
