// kernel/src/task/context_switch.rs
//! Context switching routines for task management
//! This module provides the assembly functions needed to switch between tasks

use core::arch::asm;
use crate::task::task_structs::TaskContext;
// use core::ptr::addr_of_mut; // Unused, so removing

/// Saves the current CPU context to the provided TaskContext.
/// # Safety
/// This is unsafe because it directly manipulates hardware registers
/// and requires a valid, properly aligned TaskContext pointer.
#[inline(never)]
pub unsafe fn save_context(context: *mut TaskContext) {
    // Capture all general purpose registers, RSP, and RFLAGS
    // into the provided context structure
    asm!(
        // Save general purpose registers
        "mov [{0} + 0x00], rax", // rax
        "mov [{0} + 0x08], rbx", // rbx
        "mov [{0} + 0x10], rcx", // rcx
        "mov [{0} + 0x18], rdx", // rdx
        "mov [{0} + 0x20], rsi", // rsi
        "mov [{0} + 0x28], rdi", // rdi
        "mov [{0} + 0x30], rbp", // rbp
        "mov [{0} + 0x38], r8",  // r8
        "mov [{0} + 0x40], r9",  // r9
        "mov [{0} + 0x48], r10", // r10
        "mov [{0} + 0x50], r11", // r11
        "mov [{0} + 0x58], r12", // r12
        "mov [{0} + 0x60], r13", // r13
        "mov [{0} + 0x68], r14", // r14
        "mov [{0} + 0x70], r15", // r15
        
        // Save stack pointer - compensating for the return address pushed by this call
        "lea rax, [rsp + 8]",
        "mov [{0} + 0x78], rax", // rsp
        
        // Save instruction pointer (this will be the return address)
        "mov rax, [rsp]",
        "mov [{0} + 0x80], rax", // rip
        
        // Save RFLAGS
        "pushfq",
        "pop rax",
        "mov [{0} + 0x88], rax", // rflags
        
        in(reg) context,
        // All registers clobbered except r15 (used for context ptr) and rsp
        clobber_abi("C"),
    );
}

/// Restores the CPU context from the provided TaskContext.
/// # Safety
/// This is unsafe because it directly manipulates hardware registers
/// and requires a valid, properly aligned TaskContext pointer.
/// This function does not return to the caller.
#[inline(never)]
pub unsafe fn restore_context(context: *const TaskContext) -> ! {
    // Load all registers from the TaskContext and jump to the saved RIP
    asm!(
        // Restore general purpose registers except RAX, RCX, RDX (needed for manipulation)
        "mov rbx, [{0} + 0x08]", // rbx
        "mov rsi, [{0} + 0x20]", // rsi
        "mov rdi, [{0} + 0x28]", // rdi
        "mov rbp, [{0} + 0x30]", // rbp
        "mov r8,  [{0} + 0x38]", // r8
        "mov r9,  [{0} + 0x40]", // r9
        "mov r10, [{0} + 0x48]", // r10
        "mov r11, [{0} + 0x50]", // r11
        "mov r12, [{0} + 0x58]", // r12
        "mov r13, [{0} + 0x60]", // r13
        "mov r14, [{0} + 0x68]", // r14
        "mov r15, [{0} + 0x70]", // r15
        
        // Restore RFLAGS
        "mov rax, [{0} + 0x88]",  // rflags
        "push rax",
        "popfq",
        
        // Setup stack for "ret" to instruction pointer
        "mov rsp, [{0} + 0x78]", // rsp
        "push [{0} + 0x80]",     // push rip for subsequent "ret"
        
        // Now restore the remaining registers
        "mov rax, [{0} + 0x00]", // rax
        "mov rcx, [{0} + 0x10]", // rcx
        "mov rdx, [{0} + 0x18]", // rdx
        
        // Return to the stored instruction pointer
        "ret",
        
        in(reg) context,
        options(noreturn),
    );
}

/// Switches from the current task to a new task.
/// Saves the current context to `from_context` and
/// restores the CPU state from `to_context`.
/// # Safety
/// This is unsafe for the same reasons as save_context and restore_context,
/// and also requires valid context pointers.
#[inline(never)]
pub unsafe fn switch_context(from_context: *mut TaskContext, to_context: *const TaskContext) {
    // Save current context, then restore the new one
    save_context(from_context);
    restore_context(to_context);
} 