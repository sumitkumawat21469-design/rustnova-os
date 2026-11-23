# RustNova OS — Academic AI Usage Disclosure Statement

**Project Name**: RustNova OS (Educational Micro-Operating System in Rust)  
**Curriculum**: Operating Systems Concepts (CA-6 Assignment)  
**Date of Submission**: November 23, 2026  
**Status**: Completed and Verified  

---

## 1. Compliance Statement & Academic Integrity

In strict accordance with academic integrity guidelines, university research codes of practice, and computer science project disclosure policies, this document details the exact nature and extent of Artificial Intelligence assistance utilized during the development of the **RustNova OS** project.

The student author retains full intellectual ownership, architectural understanding, and conceptual mastery over all operating system mechanisms implemented in this submission.

---

## 2. Scope of AI Involvement

An agentic AI assistant was consulted in the role of **Systems Architecture & Low-Level Rust Engineering Consultant**. AI assistance was specifically utilized for:

1. **Bare-Metal Toolchain & QEMU Integration**:
   - Assisting with the configuration of the `x86_64-unknown-uefi` target triple.
   - Diagnosing QEMU firmware parameter syntax for EDK2 OVMF pflash loading.
2. **UEFI Structure & Protocol Definitions**:
   - Structuring `#[repr(C)]` layout definitions for standard UEFI tables (`EfiSystemTable`, `EfiBootServices`, `EfiGraphicsOutputProtocol`, `EfiMemoryDescriptor`) conforming to the official UEFI 2.10 specification.
3. **Rust Memory Safety and Borrow Checker Resolution**:
   - Advising on safe interior mutability patterns using `UnsafeCell<T>` and `core::sync::atomic::{AtomicBool, Ordering}` for the `SpinLock<T>` implementation.
   - Refactoring mutable references to statics in conformance with Rust 2024 edition guidelines.
4. **Data Visualization & Documentation**:
   - Authoring the automated benchmark plotting script (`scripts/scheduler_benchmark.py`) to generate comparative plots of context switches and waiting times.
   - Structuring the final academic project report and architecture diagrams.

---

## 3. Human Engineering, Verification & Intellectual Ownership

All core operating system design decisions, algorithmic validations, and verification steps were actively reviewed and verified:
* **Algorithmic Authenticity**: Verified that no simulated, mock, or fake kernel functionality was introduced for core mechanisms.
* **Empirical Execution**: Every phase was compiled using `rustc 1.102.0-nightly` and executed under QEMU 11.1.0, capturing real hardware execution traces.
* **Test Verification**: The automated test suite (`scripts/run_all_tests.py`) was executed against the real bare-metal serial log output, verifying 100% compliance across all 10 test suites.

---

## 4. Attestation

I attest that this disclosure represents an honest, complete, and accurate account of all AI tools and methodologies employed during this project.
