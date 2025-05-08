// kernel/src/gdt.rs
use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};
use lazy_static::lazy_static;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

const STACK_SIZE: usize = 4096 * 2; // 8 KiB stack for double fault handler
#[allow(dead_code)]
struct Stack([u8; STACK_SIZE]);

// Static stack for the double fault handler
// Using `static mut` is generally discouraged, but for a raw stack pointer in TSS it's common.
// We must ensure this is only written to once during TSS setup.
static mut DOUBLE_FAULT_STACK: Stack = Stack([0; STACK_SIZE]);

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            // Safety: We are initializing the stack pointer. This static mut is only written here.
            // The stack grows downwards, so we give the address of the top of the stack.
            let stack_start = VirtAddr::from_ptr(unsafe { &raw const DOUBLE_FAULT_STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end // TSS expects the high address (top of the stack)
        };
        // We could also set tss.privilege_stack_table[0] for ring 0 stack if needed,
        // but bootloader usually handles the initial kernel stack.
        tss
    };
}

#[allow(dead_code)]
struct Selectors {
    code_selector: SegmentSelector,
    data_selector: SegmentSelector, // Though data segments are mostly unused in 64-bit mode for flat memory
    tss_selector: SegmentSelector,
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let data_selector = gdt.add_entry(Descriptor::kernel_data_segment()); // Still needed for some ops
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt, Selectors { code_selector, data_selector, tss_selector })
    };
}

/// Initializes the Global Descriptor Table and Task State Segment.
/// This function must be called before interrupts are enabled,
/// especially if the double fault handler relies on the IST.
pub fn init_gdt() {
    use x86_64::instructions::segmentation::{CS, Segment}; // Removed LoadTr

    GDT.0.load(); // Load the GDT structure itself

    unsafe {
        CS::set_reg(GDT.1.code_selector); // Reload code segment selector
        
        // Direct use of the `ltr` instruction via inline assembly
        // The 16-bit TSS selector needs to be in a 16-bit register, typically ax/bx/cx/dx
        let tss_selector = GDT.1.tss_selector.0 as u16; // Get raw selector value
        core::arch::asm!("ltr ax", in("ax") tss_selector);
    }
    crate::serial_println!("GDT and TSS initialized and loaded.");
}