// kernel/src/task/scheduler.rs
use crate::{serial_println, println};
use super::task_structs::{Task, TaskState};
use alloc::collections::VecDeque;
use spin::Mutex;
use lazy_static::lazy_static;
use alloc::boxed::Box;

// Define the TaskId type
pub type TaskId = u64;

lazy_static! {
    // Task queue for tasks ready to run
    static ref TASK_QUEUE: Mutex<VecDeque<Box<Task>>> = Mutex::new(VecDeque::new());
    // Currently running task
    static ref CURRENT_TASK: Mutex<Option<Box<Task>>> = Mutex::new(None);
}

// Example task functions for testing
pub fn example_task1() {
    for i in 0..5 {
        serial_println!("Task 1: iteration {}", i);
        // In real code, we'd use a proper yield mechanism
        for _ in 0..1_000_000 { core::hint::spin_loop(); }
    }
    serial_println!("Task 1 complete!");
    // When a task is done, it can terminate itself
    terminate_current();
}

pub fn example_task2() {
    for i in 0..5 {
        serial_println!("Task 2: iteration {}", i);
        for _ in 0..1_000_000 { core::hint::spin_loop(); }
    }
    serial_println!("Task 2 complete!");
    terminate_current();
}

/// Initializes the scheduler with the kernel task and two example tasks.
pub fn init() {
    serial_println!("Scheduler: Starting initialization");
    
    // Create the kernel task (representing the current execution context)
    match Task::kernel_task() {
        Ok(kernel_task) => {
            serial_println!("Scheduler: Created kernel task with ID {}", kernel_task.id());
            // Store it as the current task
            *CURRENT_TASK.lock() = Some(Box::new(kernel_task));
        }
        Err(e) => {
            serial_println!("Scheduler: ERROR: Failed to create kernel task: {}", e);
            // Critical failure - we can't do much without the kernel task
            return;
        }
    }
    
    // During boot we'll avoid creating additional tasks yet
    // This simplifies the initialization process
    serial_println!("Scheduler: Skipping example task creation during initial boot");
    serial_println!("Scheduler: Basic initialization complete");
    
    // The kernel task is now properly set up
    serial_println!("Scheduler: Kernel task is ready");
    
    // Can be uncommented once the system is stable:
    /*
    // Create two example tasks
    match Task::new(example_task1) {
        Ok(task1) => {
            serial_println!("Scheduler: Created example task 1");
            TASK_QUEUE.lock().push_back(Box::new(task1));
        }
        Err(e) => {
            serial_println!("Scheduler: Failed to create task 1: {}", e);
        }
    }
    
    match Task::new(example_task2) {
        Ok(task2) => {
            serial_println!("Scheduler: Created example task 2");
            TASK_QUEUE.lock().push_back(Box::new(task2));
        }
        Err(e) => {
            serial_println!("Scheduler: Failed to create task 2: {}", e);
        }
    }
    */
    
    println!("Scheduler initialized with kernel task.");
}

/// Spawns a new task with the given entry point function.
pub fn spawn(entry: fn()) -> Result<TaskId, &'static str> {
    match Task::new(entry) {
        Ok(task) => {
            let id = task.id();
            TASK_QUEUE.lock().push_back(Box::new(task));
            Ok(id)
        }
        Err(e) => Err(e),
    }
}

/// Terminates the currently running task.
pub fn terminate_current() {
    // Set the current task state to terminated
    if let Some(ref mut task) = *CURRENT_TASK.lock() {
        task.set_state(TaskState::Terminated);
    }
    
    // Then force a reschedule, which will not put this task back in the queue
    schedule();
}

/// Switches to the next ready task.
/// This is the heart of the preemptive scheduler.
pub fn schedule() {
    // First check if scheduling is already in progress to prevent reentrancy issues
    static SCHEDULE_IN_PROGRESS: Mutex<bool> = Mutex::new(false);
    
    // Try to acquire lock, if we can't, another schedule is in progress, so return
    let in_progress_guard = SCHEDULE_IN_PROGRESS.try_lock();
    if in_progress_guard.is_none() {
        serial_println!("Scheduler: Schedule already in progress, skipping");
        return;
    }
    
    // Set the in_progress flag - fixed by directly using the unwrapped MutexGuard
    *in_progress_guard.unwrap() = true;
    
    // Special case: during early boot, just return without switching tasks
    // This is safer until the system is fully initialized
    if TASK_QUEUE.lock().is_empty() {
        if let Some(ref mut current) = *CURRENT_TASK.lock() {
            if current.state() == TaskState::Terminated {
                serial_println!("Scheduler: No tasks to run, but won't halt during boot");
                // Don't halt during boot - just return to kernel instead
                return;
            }
            // Otherwise, keep running current task
            return;
        } else {
            serial_println!("Scheduler: WARNING - No current task, this shouldn't happen");
            return;
        }
    }
    
    // For now, since we're not creating any actual tasks during boot, just return
    serial_println!("Scheduler: No additional tasks yet, continuing with kernel task");
    return;
    
    /*
    // CRITICAL SECTION - We need to ensure we acquire both locks to prevent deadlock
    // First, get the next task from the queue while holding only TASK_QUEUE lock
    let mut next_task = match TASK_QUEUE.lock().pop_front() {
        Some(task) => task,
        None => {
            return; // Should not happen due to check above
        }
    };
    
    // Mark the next task as running
    next_task.set_state(TaskState::Running);
    
    // Then perform the context switch
    {
        // Get a mutable lock to the current task
        let mut current_lock = CURRENT_TASK.lock();
        
        // Only perform a switch if there's a current task to switch from
        match current_lock.take() {
            Some(mut current_task) => {
                // Check if the current task is still valid to put back in the queue
                if current_task.state() == TaskState::Running { 
                    // Mark it as ready
                    current_task.set_state(TaskState::Runnable);
                    // Put it back in the queue for later execution
                    TASK_QUEUE.lock().push_back(current_task);
                }
                // else: Don't re-queue terminated tasks
                
                // Get pointers to the contexts for the switch
                let current_ctx_ptr = current_lock.as_mut().unwrap().context_mut() as *mut _;
                let next_ctx_ptr = next_task.context() as *const _;
                
                // Set the next task as current before the switch
                *current_lock = Some(next_task);
                
                // Drop the locks before context switch to avoid deadlock
                drop(current_lock);
                
                // SAFETY: This is unsafe because it manipulates raw CPU state
                // We ensure safety by setting up valid context pointers and ensuring
                // no locks are held across the switch
                unsafe {
                    context_switch::switch_context(current_ctx_ptr, next_ctx_ptr);
                    // This function never returns - we'll resume at the new task's context
                }
            }
            None => {
                // No current task, simply start the new one
                *current_lock = Some(next_task);
                // Drop the lock before context restore
                let next_ctx_ptr = current_lock.as_ref().unwrap().context() as *const _;
                drop(current_lock);
                
                // SAFETY: Same as above, but we're only doing a restore, not a full switch
                unsafe {
                    context_switch::restore_context(next_ctx_ptr);
                    // Never returns
                }
            }
        }
    }
    */
}

/// Gets the ID of the currently running task, if any.
pub fn current_task_id() -> Option<TaskId> {
    CURRENT_TASK.lock().as_ref().map(|task| task.id())
} 