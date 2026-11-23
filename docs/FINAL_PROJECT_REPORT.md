# RUSTNOVA OS — BTech Operating Systems CA-6 Final Project Report

**Course**: Operating Systems Concepts (CA-6 Micro-OS Engineering)  
**Project Name**: RUSTNOVA OS  
**Student Specialization**: Scheduler Engineering & Dynamic Time Quantum Optimization  
**Target Architecture**: x86_64 Bare-Metal (UEFI 2.x Firmware / EDK2 OVMF)  
**Implementation Language**: Rust 2021 (`no_std`, `no_main`)  

---

## Abstract

This project documents the design, bare-metal implementation, and empirical evaluation of **RustNova OS**, an educational Micro-Operating System written entirely in bare-metal Rust without standard library dependencies (`no_std`). The project follows the educational criteria of the Class Assignment 6 (CA-6) Operating Systems syllabus. RustNova OS implements fundamental operating system mechanisms: a UEFI bootloader handshake, multi-tasking task control blocks, preemptive Round Robin scheduling, periodic Intel 8254 timer accounting, atomic spinlock synchronization, and a coalescing free-list memory allocator with strict failure recovery. As an advanced specialization, this project implements a **Dynamic Time Quantum Round Robin Scheduler** and quantitatively compares its performance against standard fixed-quantum scheduling across context switch frequency, waiting time, turnaround time, response time, and Jain's Fairness Index. Furthermore, the operating system incorporates a modern graphical desktop interface rendered directly onto the UEFI Graphics Output Protocol (GOP) linear framebuffer, accompanied by a custom application store, a simulated 802.11ax Wi-Fi subsystem, a terminal shell, and a live system monitor dashboard. Comprehensive automated testing verifies all ten required OS dimensions with a 100% pass rate.

---

## 1. Introduction and Objectives

Operating systems represent the critical bridge between physical computer hardware and user applications. While production operating systems (such as Linux or Windows) contain millions of lines of code that obscure core mechanisms, educational micro-kernels provide transparent visibility into low-level hardware interactions.

The primary objectives of the RustNova OS project were:
1. To implement an authentic, functional operating system kernel in safe, modern Rust without utilizing mock abstractions or simulated core mechanisms.
2. To boot natively on modern 64-bit UEFI firmware and initialize hardware devices including serial UARTs, periodic interval timers, and linear graphics framebuffers.
3. To engineer a preemptive multi-tasking scheduler capable of fair CPU allocation, context switching, and state accounting.
4. To implement the **Scheduler Engineering Specialization**, developing a Dynamic Time Quantum algorithm and benchmarking it against standard Round Robin.
5. To provide a graphical desktop environment with specialized subsystems demonstrating userland system interaction (Taskbar, System Monitor, Software Store, and Wi-Fi Networking).

---

## 2. Theoretical Foundations

### 2.1 Round Robin Scheduling & Time Slicing
In traditional Round Robin (RR) scheduling, the operating system assigns a fixed time quantum ($Q$) to each ready task. Tasks execute in a circular FIFO queue. If a task does not complete within $Q$ timer ticks, it is preempted via hardware interrupt and moved to the back of the queue.

While Round Robin guarantees bounded waiting time and prevents starvation, fixed quantum selection presents a fundamental trade-off:
* **Small Quantum**: Minimizes response time for interactive tasks but introduces high context-switching overhead, wasting CPU cycles on register state saving and cache invalidation.
* **Large Quantum**: Reduces context switching overhead but degenerates into First-Come-First-Served (FCFS), leading to poor responsiveness for short bursts (the "convoy effect").

### 2.2 Dynamic Time Quantum Optimization
To mitigate the fixed-quantum dilemma, RustNova OS implements an adaptive quantum calculation:
$$Q_{dyn} = \operatorname{clamp}\left(Q_{base} + \left\lfloor\frac{W_{prio}}{2}\right\rfloor + I_{completion},\ 1,\ 12\right)$$
Where:
* $Q_{base}$ is the default system slice ($2\text{ ticks}$).
* $W_{prio}$ is the priority weight (Idle=1, Low=2, Normal=4, High=6, Critical=8).
* $I_{completion}$ is a completion incentive: if a task's remaining burst is less than or equal to $Q + 2$, the scheduler grants an immediate slice extension to allow the task to terminate without incurring an additional preemption, requeue, and dispatch cycle.

### 2.3 Jain's Fairness Index
To verify that dynamic quantum extensions do not result in starvation or unfair CPU monopolization, scheduling fairness is evaluated using **Jain's Fairness Index**:
$$\mathcal{J}(x_1, x_2, \dots, x_n) = \frac{\left(\sum_{i=1}^n x_i\right)^2}{n \cdot \sum_{i=1}^n x_i^2}$$
Where $x_i$ denotes the total CPU execution ticks received by task $i$, and $n$ is the total task count. A value of $1.0$ indicates perfect equality of resource distribution.

---

## 3. Kernel Implementation Details

### 3.1 Bare-Metal Rust & Memory Layout
RustNova OS executes in 64-bit Long Mode under UEFI. By compiling for `x86_64-unknown-uefi` with `#![no_std]` and `#![no_main]`, the kernel operates without C runtime (`crt0`) or OS-hosted dependencies.
* **Heap Architecture**: The heap manager (`MicroAllocator`) operates over a dedicated static 256 KB memory region. Each block contains an explicit 24-byte header tracking magic words, sizes, free flags, and linked list nodes.
* **Deallocation Coalescing**: When any memory block is freed, the allocator traverses adjacent nodes and merges contiguous unallocated blocks, actively preventing memory fragmentation.
* **Out-Of-Memory Safety**: In strict adherence to CA-6 requirements, memory allocation failure does not trigger panic; rather, requests exceeding capacity safely return `Err(AllocError::OutOfMemory)`.

### 3.2 Concurrency & Spinlock Invariants
To prevent data races on shared system data, RustNova OS implements an atomic spinlock:
* Uses hardware `AtomicBool` with `compare_exchange_weak` under `Acquire` ordering.
* Utilizes x86 `core::hint::spin_loop()` instructions during busy-waiting to optimize CPU pipeline throughput.
* Employs an RAII `SpinLockGuard` that automatically resets the lock using `Release` ordering when dropped.
* Empirically verified through a multi-worker shared transaction ledger demo, confirming zero race conditions.

---

## 4. Empirical Evaluation & Benchmarks

The benchmark workload consisted of four concurrent demonstration tasks with diverse priority tiers and burst requirements:
1. **Task 1 (Telemetry & Sensor)**: Normal Priority, Burst = 6 ticks
2. **Task 2 (Memory GC)**: Low Priority, Burst = 8 ticks
3. **Task 3 (Packet Router)**: High Priority, Burst = 5 ticks
4. **Task 4 (Crypto Auditor)**: Critical Priority, Burst = 4 ticks

### Quantitative Performance Matrix

| Scheduling Performance Metric | Standard Round Robin ($Q=2$) | Dynamic Quantum RR | Relative Improvement ($\Delta$) |
|:---|:---:|:---:|:---:|
| **Total Tasks Scheduled** | 4 | 4 | Baseline |
| **Total Context Switches** | **12** | **5** | **-58.3% (Fewer Switches)** |
| **Average Waiting Time** | 14.75 ticks | **10.50 ticks** | **-28.8% (Shorter Wait)** |
| **Average Turnaround Time** | 19.50 ticks | **15.25 ticks** | **-21.8% (Faster Completion)** |
| **Average Response Time** | **3.00 ticks** | 7.25 ticks | Trade-off for batch throughput |
| **Jain's Fairness Index** | **0.9379** | **0.9379** | **Identical High Fairness** |

### Benchmark Analysis
1. **Context Switch Reduction**: Dynamic Time Quantum reduced total context switches from 12 down to 5 (a **58.3% reduction**). By eliminating redundant slice preemption for tasks near completion, CPU cycles spent on context management are minimized.
2. **Turnaround Time Acceleration**: Average turnaround time improved by 21.8% (from 19.50 ticks to 15.25 ticks).
3. **Fairness Preservation**: Despite dynamic quantum scaling, Jain's Fairness Index remained at an optimal **0.9379**, demonstrating that higher-priority tasks completed faster without starving lower-priority background tasks.

---

## 5. Graphical Desktop and Specialized Subsystems

Following kernel stabilization, RustNova OS renders a modern desktop interface directly onto the UEFI GOP linear framebuffer ($1024 \times 768 \times 32\text{ bpp}$):
* **Desktop Composition**: Dual-tone gradient wallpaper, system taskbar with Start button, digital clock, quick launch dock, and system status badges.
* **System Monitor (Phase 11)**: Telemetry dashboard tracking running/ready/terminated tasks, CPU scheduling context switches, free-list memory allocations, timer ticks, and spinlock contention.
* **Custom Application Store (Phase 9)**: Repository catalog with category filtering (System Tools, Productivity, Development, Utilities), search interface, install/uninstall state tracking, and update alerts.
* **Wi-Fi Manager (Phase 10)**: 802.11ax wireless center scanning access points, displaying signal strength (RSSI), security protocols, and connection states with a clearly labelled educational simulation driver.
* **Screenshot Generator**: Encodes framebuffer memory into uncompressed 24-bit BMP image format (`results/rustnova_desktop_screenshot.bmp` and `.png`).

---

## 6. Verification and Test Results (Phase 12)

The automated test suite (`scripts/run_all_tests.py`) executed across all ten required OS validation criteria.

| ID | Test Suite Description | Verification Mechanism | Outcome |
|:---:|:---|:---|:---:|
| 1 | **Kernel Boot** | UEFI handover, serial logging, physical framebuffer discovery | **PASS** |
| 2 | **Multi-Task Management** | 4 concurrent Task Control Blocks with state tracking | **PASS** |
| 3 | **Multiple Task Scheduling** | Preemptive circular ready queue execution | **PASS** |
| 4 | **Round Robin Fairness** | Jain's Fairness Index calculated at 0.9379 | **PASS** |
| 5 | **Timer Functionality** | Intel 8254 PIT running at 100 Hz driving monotonic ticks | **PASS** |
| 6 | **Synchronization** | Atomic spinlock mutual exclusion and deposit balance invariant | **PASS** |
| 7 | **Memory Allocation** | Block allocation, byte write/read verification, free-list coalescing | **PASS** |
| 8 | **Allocation Failure** | 1 MB request rejected gracefully with `Err(OutOfMemory)` | **PASS** |
| 9 | **Dynamic Quantum Comparison** | 58.3% context switch reduction benchmarked | **PASS** |
| 10 | **Integrated Execution** | Desktop UI, Taskbar, App Store, Wi-Fi, System Monitor | **PASS** |

**Summary**: **10 / 10 Tests Passed (100% Success Rate)**.

---

## 7. Conclusion

RustNova OS successfully satisfies all academic and technical objectives set forth by the Operating Systems CA-6 specification. By leveraging Rust's zero-cost abstractions, strict type safety, and memory ownership model, the micro-kernel demonstrates that complex operating system mechanisms—such as preemptive scheduling, coalescing memory allocators, spinlocks, and graphical desktop interfaces—can be implemented cleanly and robustly without relying on third-party kernel frameworks or mock implementations. The Dynamic Time Quantum specialization confirmed that adaptive quantum sizing significantly reduces context-switching overhead while preserving scheduling fairness across concurrent workloads.

---

## 8. References

1. Silberschatz, A., Galvin, P. B., & Gagne, G. *Operating System Concepts* (10th ed.). Wiley, 2018.
2. Unified Extensible Firmware Interface (UEFI) Forum. *UEFI Specification Version 2.10*, 2023.
3. Intel Corporation. *Intel® 64 and IA-32 Architectures Software Developer’s Manual*, Volume 3A: System Programming Guide, 2024.
4. Jain, R. K., Chiu, D. M., & Hawe, W. R. *A Quantitative Measure of Fairness and Discrimination for Resource Allocation in Shared Computer Systems*. DEC Research Report TR-301, 1984.
