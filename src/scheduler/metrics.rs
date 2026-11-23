//! Scheduler Evaluation Metrics.
//!
//! Computes quantitative scheduling performance metrics:
//! - Average Waiting Time
//! - Average Turnaround Time
//! - Average Response Time
//! - Total Context Switches
//! - Jain's Fairness Index

use crate::task::TaskControlBlock;

#[derive(Copy, Clone, Debug)]
pub struct SchedulerMetrics {
    pub total_tasks: usize,
    pub total_context_switches: u32,
    pub avg_wait_time: f32,
    pub avg_turnaround_time: f32,
    pub avg_response_time: f32,
    pub jains_fairness_index: f32,
    pub total_cpu_ticks: u32,
}

impl SchedulerMetrics {
    pub fn compute(tasks: &[TaskControlBlock]) -> Self {
        if tasks.is_empty() {
            return Self {
                total_tasks: 0,
                total_context_switches: 0,
                avg_wait_time: 0.0,
                avg_turnaround_time: 0.0,
                avg_response_time: 0.0,
                jains_fairness_index: 1.0,
                total_cpu_ticks: 0,
            };
        }

        let n = tasks.len() as f32;
        let mut sum_wait: f32 = 0.0;
        let mut sum_turnaround: f32 = 0.0;
        let mut sum_response: f32 = 0.0;
        let mut total_switches: u32 = 0;
        let mut total_ticks: u32 = 0;

        let mut sum_alloc: f32 = 0.0;
        let mut sum_alloc_sq: f32 = 0.0;

        for t in tasks {
            let exec = &t.execution_info;
            sum_wait += exec.wait_time as f32;
            sum_turnaround += exec.turnaround_time as f32;
            sum_response += exec.response_time as f32;
            total_switches += exec.context_switches;
            total_ticks += exec.cpu_time_executed;

            let x = exec.cpu_time_executed as f32;
            sum_alloc += x;
            sum_alloc_sq += x * x;
        }

        // Jain's Fairness Index: (sum(x_i))^2 / (n * sum(x_i^2))
        let jains = if sum_alloc_sq > 0.0 {
            (sum_alloc * sum_alloc) / (n * sum_alloc_sq)
        } else {
            1.0
        };

        Self {
            total_tasks: tasks.len(),
            total_context_switches: total_switches,
            avg_wait_time: sum_wait / n,
            avg_turnaround_time: sum_turnaround / n,
            avg_response_time: sum_response / n,
            jains_fairness_index: jains,
            total_cpu_ticks: total_ticks,
        }
    }
}
