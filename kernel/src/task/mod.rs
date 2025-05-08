pub mod task;
pub mod scheduler;
pub mod task_structs; // For Task, TaskContext, TaskState, etc.
pub mod context_switch; // Add context switching module
// Potentially later: pub mod context_switch; (for asm routines)

// Re-export key structures for convenience
pub use task_structs::{Task, TaskState, TaskContext};
// pub use scheduler::Scheduler; // This doesn't exist, so remove it
pub use context_switch::{save_context, restore_context, switch_context};

// Re-export key structures if needed later
// pub use task::Task;
// pub use scheduler::Scheduler; 