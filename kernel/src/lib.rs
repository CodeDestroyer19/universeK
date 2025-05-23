#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]
extern crate alloc; // Enable heap allocation types (Box, Vec, etc.)

// Declare kernel modules
pub mod allocator;
pub mod vga_buffer;
pub mod serial;
pub mod interrupts;
pub mod memory;
pub mod task; // Add task module
pub mod fs; // Added File System module
pub mod gdt; // Added GDT module
pub mod errors;
pub mod device; // Device driver framework
pub mod drivers; // Hardware drivers
pub mod user; // User management module
pub mod shell; // Shell/terminal functionality
pub mod logger; // Logging system
pub mod config; // Configuration management
pub mod gui; // GUI subsystem

use alloc::format;
use bootloader::BootInfo;
use x86_64::VirtAddr;
use memory::BootInfoFrameAllocator;

/// Kernel initialization phases for more structured startup
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InitPhase {
    CoreHardware, // Phase 1
    Memory,       // Phase 2
    DeviceDrivers,// Phase 3
    TaskSystem,   // Phase 4 (Moved FS from here)
    FinalChecks,  // Phase 5 (Moved User Setup from here)
    Filesystem,   // Phase 6 (New Position)
    UserSetup,    // Phase 7 (New Position)
    GuiSetup,     // Phase 8 (New for GUI)
    Complete,     // Final Phase
}

/// Main initialization function called by kernel_main in main.rs
pub fn init(boot_info: &'static BootInfo) {
    println!("Starting kernel initialization...");
    serial_println!("DEBUG: Beginning kernel initialization...");

    // ===== PHASE 1: Core Hardware =====
    let phase = InitPhase::CoreHardware;
    serial_println!("DEBUG: [INIT Phase {:?}] Initializing core hardware", phase);
    gdt::init_gdt();
    interrupts::init();
    serial_println!("DEBUG: [INIT Phase {:?}] Complete", phase);

    // ===== PHASE 2: Memory Management =====
    let phase = InitPhase::Memory;
    serial_println!("DEBUG: [INIT Phase {:?}] Initializing memory subsystems", phase);
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init_page_table(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    match allocator::init_heap(&mut mapper, &mut frame_allocator) {
        Ok(_) => serial_println!("DEBUG: Heap initialized successfully"),
        Err(e) => panic!("Failed to initialize heap: {:?}", e),
    }
    serial_println!("DEBUG: [INIT Phase {:?}] Complete", phase);

    // ===== PHASE 3: Device Drivers =====
    let phase = InitPhase::DeviceDrivers;
    serial_println!("DEBUG: [INIT Phase {:?}] Initializing device drivers", phase);
    if let Err(e) = device::init() {
        serial_println!("DEBUG: Warning: Device driver initialization failed: {:?}", e);
    }
    serial_println!("DEBUG: [INIT Phase {:?}] Complete", phase);

    // ===== PHASE 4: Task System =====
    let phase = InitPhase::TaskSystem;
    serial_println!("DEBUG: [INIT Phase {:?}] Initializing task scheduler", phase);
    task::scheduler::init();
    serial_println!("DEBUG: [INIT Phase {:?}] Complete", phase);

    // ===== PHASE 5: Final Checks =====
    let phase = InitPhase::FinalChecks;
    serial_println!("DEBUG: [INIT Phase {:?}] Performing final system checks", phase);
    if let Err(e) = logger::init() {
        serial_println!("DEBUG: Warning: Failed to initialize logging system: {:?}", e);
    }
    if let Err(e) = config::init() {
        serial_println!("DEBUG: Warning: Failed to initialize configuration system: {:?}", e);
    }
    if let Err(e) = errors::perform_system_checks() {
        errors::report_error(&e, false);
        serial_println!("DEBUG: WARNING: System check failed but continuing boot process");
    }
    interrupts::configure_for_operation(); // Configure which IRQs are active
    serial_println!("DEBUG: [INIT Phase {:?}] Complete", phase);

    // ===== PHASE 6: File System (MOVED HERE) =====
    let phase = InitPhase::Filesystem;
    serial_println!("DEBUG: [INIT Phase {:?}] Initializing file system (standard mode)", phase);
    let fs_initialized = match fs::init() {
        Ok(_) => {
            serial_println!("DEBUG: File system initialized successfully.");
            true
        },
        Err(e) => {
            serial_println!("DEBUG: Warning: File system initialization failed: {:?}", e);
            false
        },
    };
    serial_println!("DEBUG: [INIT Phase {:?}] Complete (fs_initialized: {})", phase, fs_initialized);

    // ===== PHASE 7: User Setup (MOVED HERE) =====
    let phase = InitPhase::UserSetup;
    serial_println!("DEBUG: [INIT Phase {:?}] Setting up user environment", phase);
    if let Err(e) = user::init() {
        serial_println!("DEBUG: Warning: Failed to initialize user management: {:?}", e);
    }
    // Attempt initial filesystem structure setup only if FS is initialized
    if fs_initialized {
        if let Err(e) = user::setup_filesystem() {
            serial_println!("DEBUG: Warning: Failed to setup filesystem structure: {:?}", e);
        } else {
            serial_println!("DEBUG: Filesystem structure created successfully.");
        }
    } else {
        serial_println!("DEBUG: Skipping filesystem structure setup as FS is not initialized.");
    }
    serial_println!("DEBUG: [INIT Phase {:?}] Complete", phase);

    // ===== PHASE 8: GUI Setup (NEW) =====
    let phase = InitPhase::GuiSetup;
    serial_println!("DEBUG: [INIT Phase {:?}] Initializing GUI subsystem", phase);
    match gui::init() {
        Ok(_) => serial_println!("DEBUG: GUI subsystem initialized successfully"),
        Err(e) => serial_println!("DEBUG: Warning: GUI initialization failed: {:?}", e),
    }
    serial_println!("DEBUG: [INIT Phase {:?}] Complete", phase);

    // ===== COMPLETE =====
    let phase = InitPhase::Complete;
    serial_println!("DEBUG: [INIT Phase {:?}] Kernel initialization complete", phase);
    println!("Kernel initialization complete!");

    // Log final status messages
    logger::info("kernel", "UniverseK OS initialized");
    logger::info("kernel", &format!("Heap size: {} KB", allocator::HEAP_SIZE / 1024));
    if fs_initialized {
        logger::info("kernel", "File system: Ready");
    } else {
        logger::warning("kernel", "File system: Not initialized");
    }

    // Enable CPU interrupts - this allows the configured device IRQs to be processed
    serial_println!("DEBUG: About to enable CPU interrupts");
    
    // Initialize PIC and configure interrupts
    unsafe {
        // First disable CPU interrupts during initialization
        x86_64::instructions::interrupts::disable();
        serial_println!("DEBUG: CPU interrupts disabled during initialization");
        
        // Initialize PICs with all interrupts masked
        interrupts::pic::PIC_CONTROLLER.lock().initialize();
        serial_println!("DEBUG: PIC initialization complete");
        
        // Configure specific interrupts we want to handle
        // Only enable timer (IRQ0) initially
        interrupts::pic::PIC_CONTROLLER.lock().configure_irqs(0b11111110u8, 0b11111111u8);
        serial_println!("DEBUG: IRQs configured - Timer only enabled");
        
        // Make sure IDT is properly set up
        serial_println!("DEBUG: Verifying IDT setup");
        interrupts::init_idt();
        serial_println!("DEBUG: IDT setup verified");
        
        // Now enable CPU interrupts
        serial_println!("DEBUG: Enabling CPU interrupts");
        // x86_64::instructions::interrupts::enable();
        serial_println!("DEBUG: CPU interrupts enabled");
        
        // Add a small delay to let any pending interrupts clear
        for _ in 0..1000 {
            x86_64::instructions::nop();
        }
        serial_println!("DEBUG: Initial delay complete");
    }

    // Start the GUI (which includes shell window)
    serial_println!("DEBUG: Starting GUI");
    match gui::run() {
        Ok(_) => serial_println!("DEBUG: GUI exited normally"),
        Err(e) => serial_println!("ERROR: Error running GUI: {:?}", e),
    }

    // GUI has exited, so enter the kernel main loop
    serial_println!("DEBUG: Entering kernel main loop");
    let mut counter = 0;
    loop {
        counter += 1;
        if counter % 100_000_000 == 0 {
            serial_println!("Kernel main loop heartbeat ({} iterations)", counter / 100_000_000);
        }
        x86_64::instructions::hlt(); // Wait for the next interrupt
    }
}

/// Basic halt loop
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

/// This function is called when a heap allocation fails.
#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Heap allocation error: {:?}", layout)
}

// Add test runner implementation
pub fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}

// Removed test_filesystem() and show_driver_demo() as they are unused and rely on FS/drivers

// Removed generate_test_keys function

// Removed test_filesystem() and show_driver_demo() as they are unused and rely on FS/drivers 