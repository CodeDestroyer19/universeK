// kernel/src/task/task.rs
use core::sync::atomic::{AtomicU64, Ordering};
use crate::task::task_structs::{TaskContext, TaskState};
use alloc::boxed::Box;
use x86_64::VirtAddr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskId(u64);

impl TaskId {
    // Make new public for scheduler usage
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        // Start IDs from 1 for clarity (0 could be reserved for idle/kernel)
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed) + 1)
    }
}

// Use the enhanced TaskState from task_structs now
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum TaskState {
//    Ready,  // Can be scheduled
//    Running, // Currently executing
//    Blocked, // Waiting for an event
//    // Terminated, // Finished execution (maybe later)
// }

#[derive(Debug)]
#[allow(dead_code)]
pub struct Task {
    id: TaskId,
    state: TaskState,
    context: TaskContext,
    kernel_stack: Box<[u8]>, // Each task has its own kernel stack
    entry_point: Option<fn()>, // Optional since kernel task doesn't need this
}

// Constants for task creation
const KERNEL_STACK_SIZE: usize = 4096 * 4; // 16 KiB stack for tasks

impl Task {
    // New task creation function that allocates a stack and sets up the context
    pub fn new(entry: fn()) -> Result<Self, &'static str> {
        use alloc::vec::Vec;

        // Allocate stack memory from the kernel heap
        let mut stack_mem = Vec::new();
        if stack_mem.try_reserve_exact(KERNEL_STACK_SIZE).is_err() {
            return Err("Failed to reserve memory for kernel stack");
        }
        stack_mem.resize(KERNEL_STACK_SIZE, 0);
        let kernel_stack = stack_mem.into_boxed_slice();

        // Calculate the stack top (stacks grow downwards on x86_64)
        let stack_top_addr = VirtAddr::from_ptr(kernel_stack.as_ptr()) + kernel_stack.len();
        
        // The task will start at the entry function
        let entry_point_addr = VirtAddr::new(entry as u64);

        // Create context with instruction pointer set to the entry function and
        // stack pointer set to the top of the allocated stack
        let context = TaskContext::new(entry_point_addr, stack_top_addr);

        Ok(Task {
            id: TaskId::new(),
            state: TaskState::Runnable, // Use TaskState::Runnable from task_structs
            context,
            kernel_stack,
            entry_point: Some(entry),
        })
    }

    // Create a simple task for the kernel's initial execution environment
    pub fn kernel_task() -> Result<Self, &'static str> {
        // For the kernel task, we use a custom-allocated stack since it's special
        // It's already running, so its context isn't important for startup,
        // but we need to be able to save its state later for context switching
        
        // Create a dummy stack just for the structure
        use alloc::vec::Vec;
        let mut stack_mem = Vec::new();
        if stack_mem.try_reserve_exact(KERNEL_STACK_SIZE).is_err() {
            return Err("Failed to reserve memory for kernel stack");
        }
        stack_mem.resize(KERNEL_STACK_SIZE, 0);
        let kernel_stack = stack_mem.into_boxed_slice();

        // Note: For the kernel task, we're already executing, so our actual RSP
        // is not the one at the top of this stack. The context will be updated
        // when the scheduler actually switches away from this task.
        let rsp = VirtAddr::new(0); // Will be filled when context is saved
        let rip = VirtAddr::new(0); // Will be filled when context is saved
        
        let context = TaskContext::new(rip, rsp);

        Ok(Task {
            id: TaskId(0), // Reserve ID 0 for the kernel task
            state: TaskState::Running,
            context,
            kernel_stack,
            entry_point: None, // No entry point for kernel task, it's already running
        })
    }

    pub fn id(&self) -> TaskId {
        self.id
    }

    pub fn state(&self) -> TaskState {
        self.state
    }

    pub fn set_state(&mut self, new_state: TaskState) {
        self.state = new_state;
    }
    
    // Get a mutable reference to the context for saving/restoring
    pub fn context_mut(&mut self) -> &mut TaskContext {
        &mut self.context
    }
    
    // Get a reference to the context for reading
    pub fn context(&self) -> &TaskContext {
        &self.context
    }
    
    // We'll need access to the entry point for task initialization
    pub fn entry_point(&self) -> Option<fn()> {
        self.entry_point
    }
} 