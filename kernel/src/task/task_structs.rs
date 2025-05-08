use core::sync::atomic::{AtomicU64, Ordering};
use alloc::boxed::Box;
use alloc::vec::Vec;
use x86_64::VirtAddr;

/// Represents the state of a task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Runnable,  // Ready to run
    Running,   // Currently executing
    Blocked,   // Waiting for an event (e.g., I/O, semaphore)
    Terminated, // Task has finished execution
}

/// Represents the CPU context of a task.
/// This structure needs to be `#[repr(C)]` to ensure a defined layout
/// for assembly context switching code.
/// For simplicity, we'll start with a few key registers.
/// `rbp` and `rip` are implicitly managed by function calls/returns initially,
/// but for a full context switch, they (and others) need explicit saving/restoring.
/// `rsp` is crucial.
#[derive(Debug, Clone)]
#[repr(C)]
pub struct TaskContext {
    // General purpose registers (order can matter for asm)
    // We will refine this list as we implement the context switch
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,

    // Stack pointer
    pub rsp: u64,

    // Instruction pointer
    pub rip: u64,

    // RFLAGS register
    pub rflags: u64,

    // We might also need to save/restore segment registers (cs, ds, es, fs, gs, ss)
    // and CR3 (page table base) for tasks in different address spaces.
    // For now, kernel tasks share the same address space.
}

impl TaskContext {
    /// Creates a new, default (mostly zeroed) context.
    /// The `rip` and `rsp` must be set appropriately before this context can be run.
    pub fn new(rip: VirtAddr, rsp: VirtAddr) -> Self {
        TaskContext {
            rax: 0, rbx: 0, rcx: 0, rdx: 0, rsi: 0, rdi: 0, rbp: rsp.as_u64(), // Convention: rbp points to base of stack frame
            r8: 0, r9: 0, r10: 0, r11: 0, r12: 0, r13: 0, r14: 0, r15: 0,
            rsp: rsp.as_u64(),
            rip: rip.as_u64(),
            rflags: 0x202, // Initialize RFLAGS: IF=1 (interrupts enabled), bit 1 is always 1.
        }
    }
}

/// Represents a single task in the system.
#[allow(dead_code)]
pub struct Task {
    id: u64,
    state: TaskState,
    context: TaskContext,
    kernel_stack: Box<[u8]>, // Each task has its own kernel stack
    // The actual entry function for the task
    entry_point: fn(),
}

// For generating unique task IDs
static NEXT_TASK_ID: AtomicU64 = AtomicU64::new(0);

impl Task {
    /// Creates a special kernel task representing the initial execution context.
    /// This is used to bootstrap the scheduler.
    pub fn kernel_task() -> Result<Self, &'static str> {
        let id = NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed);
        
        // For the kernel task, we don't allocate a new stack - it's already running on one.
        // However, we still need a placeholder allocation for the Task structure
        let kernel_stack = Box::new([0u8; 8]); // Minimal placeholder
        
        // We'll use dummy values for the kernel task context initially
        // These will get replaced with real values on the first context switch
        let dummy_addr = VirtAddr::new(0xDEADBEEF);
        let context = TaskContext::new(dummy_addr, dummy_addr);
        
        // The special property of the kernel task is that it's already running,
        // so its actual context will be saved during the first context switch
        Ok(Task {
            id,
            state: TaskState::Running,  // Important: kernel task starts as Running
            context,
            kernel_stack,
            entry_point: || {}, // Dummy fn pointer, never used
        })
    }

    /// Creates a new task with a given entry point.
    /// The entry_point is a function pointer `fn()` where the task will begin execution.
    /// This function will allocate a kernel stack for the new task.
    pub fn new(entry: fn()) -> Result<Self, &'static str> {
        let id = NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed);

        // Allocate stack memory from the kernel heap.
        let mut stack_mem = Vec::new();
        if stack_mem.try_reserve_exact(DEFAULT_KERNEL_STACK_SIZE).is_err() {
            return Err("Failed to reserve memory for kernel stack");
        }
        // Initialize with a pattern or zeros for debugging if desired, then into_boxed_slice
        stack_mem.resize(DEFAULT_KERNEL_STACK_SIZE, 0);
        let kernel_stack = stack_mem.into_boxed_slice();

        // Calculate the stack top. Stacks grow downwards.
        // The `rsp` should point to the highest address of the allocated stack memory.
        let stack_top_addr = VirtAddr::from_ptr(kernel_stack.as_ptr()) + kernel_stack.len();
        
        // The initial instruction pointer will point to a wrapper that calls the entry function.
        // For now, let's point directly to the entry function for simplicity.
        // Later, we might introduce `task_entry_wrapper(entry_fn: fn())`
        let entry_point_addr = VirtAddr::new(entry as u64);

        Ok(Task {
            id,
            state: TaskState::Runnable,
            context: TaskContext::new(entry_point_addr, stack_top_addr),
            kernel_stack,
            entry_point: entry,
        })
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn state(&self) -> TaskState {
        self.state
    }

    pub fn set_state(&mut self, new_state: TaskState) {
        self.state = new_state;
    }
    
    // Getter for context (immutable)
    pub fn context(&self) -> &TaskContext {
        &self.context
    }
    
    // Getter for context (mutable for scheduler to update RSP/RIP upon context switch)
    pub fn context_mut(&mut self) -> &mut TaskContext {
        &mut self.context
    }
}

// We also need to consider how stacks are allocated and managed.
// For kernel tasks, they can be allocated from the kernel heap.
// A typical stack size might be 4KiB or 8KiB.

pub const DEFAULT_KERNEL_STACK_SIZE: usize = 4096 * 2; // 8 KiB stack 