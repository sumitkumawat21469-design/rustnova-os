//! Phase 7: Advanced Specialization - Dynamic Time Quantum Round Robin.
//!
//! Extends traditional Round Robin by computing task-specific dynamic time slices
//! based on task priority weights, remaining CPU burst ratio, and completion thresholds.
//!
//! Objectives:
//! - Reduce unnecessary context switch overhead.
//! - Lower average waiting time and turnaround time for mixed workloads.
//! - Preserve fairness using weighted time credits.

use crate::println;
use crate::scheduler::metrics::SchedulerMetrics;
use crate::scheduler::queue::{ReadyQueue, MAX_TASKS};
use crate::task::{TaskControlBlock, TaskState};

pub struct DynamicQuantumScheduler {
    pub tasks: [Option<TaskControlBlock>; MAX_TASKS],
    pub queue: ReadyQueue,
    pub active_task_id: Option<u32>,
    pub base_quantum: u32,
    pub current_dynamic_quantum: u32,
    pub slice_counter: u32,
    pub total_switches: u32,
}

impl DynamicQuantumScheduler {
    pub const fn new(base_quantum: u32) -> Self {
        Self {
            tasks: [None; MAX_TASKS],
            queue: ReadyQueue::new(),
            active_task_id: None,
            base_quantum,
            current_dynamic_quantum: base_quantum,
            slice_counter: 0,
            total_switches: 0,
        }
    }

    pub fn add_task(&mut self, task: TaskControlBlock) -> bool {
        let id = task.id as usize;
        if id >= MAX_TASKS {
            return false;
        }
        self.tasks[id] = Some(task);
        self.queue.push(task.id);
        true
    }

    /// Computes Dynamic Time Quantum for a task:
    /// Q_dyn = base + (priority_weight / 2) + completion_incentive
    pub fn compute_task_quantum(base_quantum: u32, task: &TaskControlBlock) -> u32 {
        let priority_bonus = task.priority.weight() / 2;
        let mut q = base_quantum + priority_bonus;

        // Completion incentive: if remaining burst <= q * 1.5, extend quantum
        // to let the task terminate immediately instead of incurring a context switch
        let rem = task.execution_info.cpu_burst_remaining;
        if rem > 0 && rem <= (q + 2) {
            q = rem;
        }

        q.clamp(1, 12)
    }

    fn switch_task(&mut self, current_tick: u64) {
        if let Some(active_id) = self.active_task_id {
            if let Some(task) = &mut self.tasks[active_id as usize] {
                if task.state == TaskState::Running {
                    task.mark_ready();
                    self.queue.push(active_id);
                }
            }
        }

        if let Some(next_id) = self.queue.pop() {
            let base_q = self.base_quantum;
            if let Some(task) = &mut self.tasks[next_id as usize] {
                let dynamic_q = Self::compute_task_quantum(base_q, task);
                task.assigned_quantum = dynamic_q;
                task.remaining_quantum = dynamic_q;
                task.mark_running(current_tick);

                self.active_task_id = Some(next_id);
                self.current_dynamic_quantum = dynamic_q;
                self.slice_counter = 0;
                self.total_switches += 1;

                println!(
                    "Task {} → Running [Dynamic Quantum: {} ticks, Priority: {}]",
                    task.id,
                    dynamic_q,
                    task.priority.as_str()
                );
            }
        } else {
            self.active_task_id = None;
        }
    }

    pub fn tick(&mut self, current_tick: u64) -> bool {
        for slot in self.tasks.iter_mut() {
            if let Some(task) = slot {
                if task.state == TaskState::Ready {
                    task.increment_wait_time();
                }
            }
        }

        if let Some(active_id) = self.active_task_id {
            let mut completed = false;
            if let Some(task) = &mut self.tasks[active_id as usize] {
                self.slice_counter += 1;
                completed = task.tick_execute(current_tick);
            }

            if completed {
                if let Some(task) = &self.tasks[active_id as usize] {
                    println!(
                        "   [DYNAMIC SCHEDULER] Task {} completed in {} ticks (wait: {}, turnaround: {})",
                        task.id,
                        task.execution_info.cpu_time_executed,
                        task.execution_info.wait_time,
                        task.execution_info.turnaround_time
                    );
                }
                self.active_task_id = None;
                self.switch_task(current_tick);
            } else if self.slice_counter >= self.current_dynamic_quantum {
                self.switch_task(current_tick);
            }
        } else {
            self.switch_task(current_tick);
        }

        self.has_active_tasks()
    }

    pub fn has_active_tasks(&self) -> bool {
        self.active_task_id.is_some() || !self.queue.is_empty()
    }

    pub fn collect_metrics(&self) -> SchedulerMetrics {
        let mut list: [TaskControlBlock; MAX_TASKS] = [TaskControlBlock::new(
            0,
            "",
            crate::task::TaskPriority::Normal,
            0,
            0,
            None,
        ); MAX_TASKS];
        let mut count = 0;

        for slot in self.tasks.iter() {
            if let Some(task) = slot {
                list[count] = *task;
                count += 1;
            }
        }

        SchedulerMetrics::compute(&list[..count])
    }
}
