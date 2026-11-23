//! Standard Round Robin Preemptive Scheduler.
//!
//! Implements a circular Ready Queue with fixed Time Quantum slices,
//! task switching, CPU time accounting, and formatted scheduling logs.

use crate::println;
use crate::scheduler::metrics::SchedulerMetrics;
use crate::scheduler::queue::{ReadyQueue, MAX_TASKS};
use crate::task::{TaskControlBlock, TaskState};

pub struct RoundRobinScheduler {
    pub tasks: [Option<TaskControlBlock>; MAX_TASKS],
    pub queue: ReadyQueue,
    pub active_task_id: Option<u32>,
    pub time_quantum: u32,
    pub slice_counter: u32,
    pub total_switches: u32,
}

impl RoundRobinScheduler {
    pub const fn new(time_quantum: u32) -> Self {
        Self {
            tasks: [None; MAX_TASKS],
            queue: ReadyQueue::new(),
            active_task_id: None,
            time_quantum,
            slice_counter: 0,
            total_switches: 0,
        }
    }

    /// Adds a task to the scheduler and places it in the Ready Queue.
    pub fn add_task(&mut self, task: TaskControlBlock) -> bool {
        let id = task.id as usize;
        if id >= MAX_TASKS {
            return false;
        }
        self.tasks[id] = Some(task);
        self.queue.push(task.id);
        true
    }

    /// Selects and switches to the next task in the ready queue.
    fn switch_task(&mut self, current_tick: u64) {
        // If there was an active task that didn't terminate, place it back into the ready queue
        if let Some(active_id) = self.active_task_id {
            if let Some(task) = &mut self.tasks[active_id as usize] {
                if task.state == TaskState::Running {
                    task.mark_ready();
                    self.queue.push(active_id);
                }
            }
        }

        // Dequeue next ready task
        if let Some(next_id) = self.queue.pop() {
            if let Some(task) = &mut self.tasks[next_id as usize] {
                task.mark_running(current_tick);
                self.active_task_id = Some(next_id);
                self.slice_counter = 0;
                self.total_switches += 1;

                // Required Phase 3 log format: Task X → Running
                println!("Task {} → Running", task.id);
            }
        } else {
            self.active_task_id = None;
        }
    }

    /// Executes one timer tick step.
    /// Returns `true` if any tasks are still running/ready.
    pub fn tick(&mut self, current_tick: u64) -> bool {
        // Advance waiting time for all tasks waiting in Ready state
        for slot in self.tasks.iter_mut() {
            if let Some(task) = slot {
                if task.state == TaskState::Ready {
                    task.increment_wait_time();
                }
            }
        }

        // Check if an active task exists
        if let Some(active_id) = self.active_task_id {
            let mut completed = false;
            if let Some(task) = &mut self.tasks[active_id as usize] {
                self.slice_counter += 1;
                completed = task.tick_execute(current_tick);
            }

            if completed {
                if let Some(task) = &self.tasks[active_id as usize] {
                    println!(
                        "   [SCHEDULER] Task {} completed in {} ticks (wait: {}, turnaround: {})",
                        task.id,
                        task.execution_info.cpu_time_executed,
                        task.execution_info.wait_time,
                        task.execution_info.turnaround_time
                    );
                }
                self.active_task_id = None;
                self.switch_task(current_tick);
            } else if self.slice_counter >= self.time_quantum {
                // Quantum expired: trigger preemptive context switch
                self.switch_task(current_tick);
            }
        } else {
            // No active task currently running, pick from queue
            self.switch_task(current_tick);
        }

        self.has_active_tasks()
    }

    /// Returns true if any task is not yet terminated.
    pub fn has_active_tasks(&self) -> bool {
        self.active_task_id.is_some() || !self.queue.is_empty()
    }

    /// Collects execution records for all tasks.
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
