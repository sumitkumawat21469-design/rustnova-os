//! Synchronization Demonstration.
//!
//! Demonstrates shared resource access, atomic lock acquisition, safe mutation
//! inside critical sections, lock release, and prevention of race conditions.

use crate::println;
use crate::sync::spinlock::SpinLock;

/// Shared Resource: High-Security Transaction Ledger.
pub struct SharedLedger {
    pub account_balance: u64,
    pub transaction_count: u32,
    pub checksum: u64,
}

impl SharedLedger {
    pub const fn new(initial_balance: u64) -> Self {
        Self {
            account_balance: initial_balance,
            transaction_count: 0,
            checksum: initial_balance ^ 0xAA55AA55,
        }
    }

    pub fn deposit(&mut self, amount: u64, actor: &str) {
        let old = self.account_balance;
        self.account_balance += amount;
        self.transaction_count += 1;
        self.checksum = self.account_balance ^ 0xAA55AA55;
        println!(
            "   [SYNC-LEDGER] {} performed deposit +${}: balance ${} -> ${} (tx #{})",
            actor, amount, old, self.account_balance, self.transaction_count
        );
    }
}

pub static GLOBAL_LEDGER: SpinLock<SharedLedger> = SpinLock::new(SharedLedger::new(1000));

/// Runs Phase 5 Synchronization demonstration.
pub fn run_synchronization_demo() -> bool {
    println!("\n\x1b[33m--- PHASE 5: SYNCHRONIZATION DEMONSTRATION ---\x1b[0m");
    println!("[SYNC] Initial state: Lock is free, balance = $1000");

    // Task 1 attempts acquisition
    println!("[SYNC] Worker-1 attempting lock acquisition...");
    {
        let mut guard1 = GLOBAL_LEDGER.lock();
        println!("[SYNC] Worker-1 ACQUIRED lock successfully!");
        guard1.deposit(250, "Worker-1");

        // While locked, check try_lock from competing Worker-2
        println!("[SYNC] Worker-2 attempting try_lock while Worker-1 holds lock...");
        let try_res = GLOBAL_LEDGER.try_lock();
        match try_res {
            Some(_) => {
                println!("[SYNC-FAIL] Race condition detected: Worker-2 obtained lock concurrently!");
                return false;
            }
            None => {
                println!("[SYNC-SUCCESS] Mutual exclusion verified: Worker-2 correctly denied concurrent access!");
            }
        }

        println!("[SYNC] Worker-1 completing critical section and releasing lock...");
    } // guard1 drops here -> lock released

    // Now Worker-2 acquires lock
    println!("[SYNC] Worker-2 attempting lock acquisition after release...");
    {
        let mut guard2 = GLOBAL_LEDGER.lock();
        println!("[SYNC] Worker-2 ACQUIRED lock successfully!");
        guard2.deposit(500, "Worker-2");
        println!("[SYNC] Worker-2 releasing lock...");
    } // guard2 drops here

    let final_guard = GLOBAL_LEDGER.lock();
    let expected = 1000 + 250 + 500;
    let actual = final_guard.account_balance;
    let passes = actual == expected && final_guard.transaction_count == 2;

    println!(
        "[SYNC] Invariant Verification: Expected=${}, Actual=${}, TxCount={}",
        expected, actual, final_guard.transaction_count
    );
    if passes {
        println!("\x1b[32m[SYNC-PASSED] Synchronization test PASSED with 100% integrity.\x1b[0m\n");
    } else {
        println!("\x1b[31m[SYNC-FAILED] Invariant violation!\x1b[0m\n");
    }

    passes
}
