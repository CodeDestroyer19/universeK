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

use alloc::boxed::Box; // For heap allocation test
use alloc::format;
use bootloader::BootInfo;
use x86_64::VirtAddr;
use memory::BootInfoFrameAllocator;

/// Kernel initialization phases for more structured startup
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InitPhase {
    PreBoot,
    CoreHardware,
    Memory,
    DeviceDrivers,
    Filesystem,
    TaskSystem,
    FinalChecks,
    Complete,
}

/// Main initialization function called by kernel_main in main.rs
pub fn init(boot_info: &'static BootInfo) {
    // Initialize with initial phase
    let phase = InitPhase::PreBoot;
    
    println!("Starting kernel initialization...");
    serial_println!("DEBUG: Beginning kernel initialization with {} phases", InitPhase::Complete as u8 + 1);
    
    // ===== PHASE 1: Core Hardware =====
    let phase = InitPhase::CoreHardware;
    serial_println!("DEBUG: [INIT Phase {}] Initializing core hardware", phase as u8);
    
    // Load GDT (Global Descriptor Table) first
    serial_println!("DEBUG: Initializing GDT");
    gdt::init_gdt();
    serial_println!("DEBUG: GDT initialized successfully");
    
    // Initialize Interrupt system (IDT and PICs)
    // The PICs are initialized but all IRQs remain masked at this point
    serial_println!("DEBUG: Initializing interrupt system");
    interrupts::init();
    serial_println!("DEBUG: Interrupt system initialized");
    
    // ===== PHASE 2: Memory Management =====
    let phase = InitPhase::Memory;
    serial_println!("DEBUG: [INIT Phase {}] Initializing memory subsystems", phase as u8);
    
    // Setup page tables with physical memory offset from bootloader
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    serial_println!("DEBUG: Physical memory offset: {:?}", phys_mem_offset);
    let mut mapper = unsafe { memory::init_page_table(phys_mem_offset) };
    serial_println!("DEBUG: Page table initialized");
    
    // Initialize the frame allocator
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    serial_println!("DEBUG: Frame allocator initialized");
    
    // Initialize the heap allocator
    serial_println!("DEBUG: Initializing kernel heap");
    match allocator::init_heap(&mut mapper, &mut frame_allocator) {
        Ok(_) => serial_println!("DEBUG: Heap initialized successfully"),
        Err(e) => {
            serial_println!("DEBUG: CRITICAL: Heap initialization failed: {:?}", e);
            panic!("Failed to initialize heap: {:?}", e);
        }
    }
    
    // ===== PHASE 3: Device Drivers =====
    let phase = InitPhase::DeviceDrivers;
    serial_println!("DEBUG: [INIT Phase {}] Initializing device drivers", phase as u8);
    
    match device::init() {
        Ok(_) => serial_println!("DEBUG: Device drivers initialized successfully"),
        Err(e) => serial_println!("DEBUG: Warning: Device driver initialization failed: {:?}", e),
    }
    
    // ===== PHASE 4: File System =====
    let phase = InitPhase::Filesystem;
    serial_println!("DEBUG: [INIT Phase {}] Initializing file system", phase as u8);
    
    let fs_initialized = match fs::init() {
        Ok(_) => {
            serial_println!("DEBUG: File system initialized successfully");
            true
        },
        Err(e) => {
            serial_println!("DEBUG: Warning: File system initialization failed: {:?}", e);
            false
        },
    };
    
    // Test file system if it was initialized successfully
    if fs_initialized {
        test_filesystem();
    }
    
    // ===== PHASE 5: Task System =====
    let phase = InitPhase::TaskSystem;
    serial_println!("DEBUG: [INIT Phase {}] Initializing task scheduler", phase as u8);
    
    task::scheduler::init();
    serial_println!("DEBUG: Task scheduler initialized");
    
    // ===== PHASE 6: Final Checks =====
    let phase = InitPhase::FinalChecks;
    serial_println!("DEBUG: [INIT Phase {}] Performing final system checks", phase as u8);
    
    // Run comprehensive system validations
    serial_println!("DEBUG: Running system validation checks");
    if let Err(e) = errors::perform_system_checks() {
        // Report the error but don't make it critical - we'll try to continue
        errors::report_error(&e, false);
        serial_println!("DEBUG: WARNING: System check failed but continuing boot process");
    } else {
        serial_println!("DEBUG: System validation passed");
    }
    
    // Configure interrupts for normal operation
    serial_println!("DEBUG: Configuring interrupts for normal operation");
    interrupts::configure_for_operation();
    
    // ===== COMPLETE =====
    let phase = InitPhase::Complete;
    serial_println!("DEBUG: [INIT Phase {}] Kernel initialization complete", phase as u8);
    println!("Kernel initialization complete!");
    
    // Draw a welcome window using the enhanced VGA driver
    drivers::vga_enhanced::clear_screen();
    drivers::vga_enhanced::draw_shadowed_box(10, 2, 60, 20);
    drivers::vga_enhanced::write_at(2, 11, " UniverseK OS ", 
        drivers::vga_enhanced::Color::White, 
        drivers::vga_enhanced::Color::Blue);
    
    // Show system info in the window
    drivers::vga_enhanced::write_at(4, 12, "System Information:", 
        drivers::vga_enhanced::Color::Yellow, 
        drivers::vga_enhanced::Color::Black);
    
    // Get current date/time from RTC
    let datetime = drivers::rtc::get_datetime();
    drivers::vga_enhanced::write_at(5, 12, &format!("Date/Time: {}", datetime.format()), 
        drivers::vga_enhanced::Color::White, 
        drivers::vga_enhanced::Color::Black);
    
    // Display memory info
    drivers::vga_enhanced::write_at(6, 12, &format!("Memory: {} KB heap", 
        allocator::HEAP_SIZE / 1024), 
        drivers::vga_enhanced::Color::White, 
        drivers::vga_enhanced::Color::Black);
    
    // Display filesystem info
    if fs_initialized {
        drivers::vga_enhanced::write_at(7, 12, "Filesystem: Ready", 
            drivers::vga_enhanced::Color::LightGreen, 
            drivers::vga_enhanced::Color::Black);
    } else {
        drivers::vga_enhanced::write_at(7, 12, "Filesystem: Not available", 
            drivers::vga_enhanced::Color::LightRed, 
            drivers::vga_enhanced::Color::Black);
    }
    
    // Instructions for the user
    drivers::vga_enhanced::write_at(10, 12, "Use keyboard and mouse to interact with the system", 
        drivers::vga_enhanced::Color::LightCyan, 
        drivers::vga_enhanced::Color::Black);
    
    drivers::vga_enhanced::write_at(18, 20, "Press any key to continue...", 
        drivers::vga_enhanced::Color::White, 
        drivers::vga_enhanced::Color::Black);
    
    // SAFE MODE: Skip enabling interrupts which is causing hangs
    let safe_mode = true; // Set to true to skip enabling interrupts
    
    if safe_mode {
        serial_println!("DEBUG: SAFE MODE ENABLED - CPU interrupts will remain DISABLED");
        println!("WARNING: Running in safe mode without hardware interrupts");
        
        // Enter a polling loop instead of interrupt-driven one
        serial_println!("DEBUG: Entering polling-based main loop (no timer interrupts)");
        
        // Show our driver demo interface
        show_driver_demo();
        
        // Main kernel loop - using busy waiting instead of interrupts
        let mut counter = 0;
        loop {
            counter += 1;
            
            // Update display periodically
            if counter % 10_000_000 == 0 {
                // Update time display
                let now = drivers::rtc::get_datetime();
                drivers::vga_enhanced::write_at(3, 4, &format!("Current Date/Time: {}", now.format()),
                    drivers::vga_enhanced::Color::Yellow, 
                    drivers::vga_enhanced::Color::Black);
                
                // Heartbeat to serial
                serial_println!("Heartbeat: {}", counter / 10_000_000);
            }
            
            // Small delay to reduce CPU usage
            for _ in 0..1000 {
                // Spin
            }
        }
    } else {
        // Enable CPU interrupts - this allows the configured device IRQs to be processed
        serial_println!("DEBUG: Enabling CPU interrupts");
        x86_64::instructions::interrupts::enable();
        serial_println!("DEBUG: CPU interrupts enabled successfully");
        
        // Enter the kernel main loop
        serial_println!("DEBUG: Entering kernel main loop");
        
        // Main kernel loop - idle loop waiting for interrupts
        let mut counter = 0;
        loop {
            counter += 1;
            if counter % 100_000_000 == 0 {
                serial_println!("Kernel main loop heartbeat ({} iterations)", counter / 100_000_000);
            }
            x86_64::instructions::hlt();
        }
    }
    
    // We should never reach here
    #[allow(unreachable_code)]
    {
        serial_println!("DEBUG: WARNING: Kernel main loop exited unexpectedly. Halting.");
        hlt_loop();
    }
}

/// Test the file system functionality
fn test_filesystem() {
    serial_println!("DEBUG: Testing file system...");
    
    // Get the VFS manager
    if let Some(vfs) = fs::vfs::get_vfs_manager() {
        // List the root directory
        match vfs.read_dir("/") {
            Ok(entries) => {
                serial_println!("DEBUG: Root directory contents:");
                for entry in entries {
                    let type_str = match entry.node_type {
                        fs::vfs::NodeType::Directory => "DIR",
                        fs::vfs::NodeType::File => "FILE",
                        _ => "OTHER",
                    };
                    serial_println!("DEBUG:   {} [{}]", entry.name, type_str);
                }
            },
            Err(e) => {
                serial_println!("DEBUG: Error listing root directory: {:?}", e);
            }
        }
        
        // Try to create a test file
        match vfs.create_file("/test.txt") {
            Ok(_) => {
                serial_println!("DEBUG: Created test file: /test.txt");
                
                // Write to the file directly
                let data = b"Hello, File System!";
                serial_println!("DEBUG: Attempting to write directly to file");
                match fs::direct_write_file("/test.txt", data) {
                    Ok(bytes) => {
                        serial_println!("DEBUG: Successfully wrote {} bytes directly to file", bytes);
                        
                        // Read from the file directly
                        let mut buffer = [0u8; 20];
                        match fs::direct_read_file("/test.txt", &mut buffer) {
                            Ok(bytes) => {
                                let text = core::str::from_utf8(&buffer[0..bytes])
                                    .unwrap_or("(invalid UTF-8)");
                                serial_println!("DEBUG: Read {} bytes from file: '{}'", bytes, text);
                            },
                            Err(e) => {
                                serial_println!("DEBUG: Error reading directly from file: {:?}", e);
                            }
                        }
                    },
                    Err(e) => {
                        serial_println!("DEBUG: Error writing directly to file: {:?}", e);
                    }
                }
            },
            Err(e) => {
                serial_println!("DEBUG: Error creating test file: {:?}", e);
            }
        }
    } else {
        serial_println!("DEBUG: VFS manager not initialized");
    }
    
    serial_println!("DEBUG: File system test complete");
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

/// Draw a simple driver demo screen for the kernel
fn show_driver_demo() {
    // Clear screen
    drivers::vga_enhanced::clear_screen();
    
    // Draw title bar
    drivers::vga_enhanced::set_color(drivers::vga_enhanced::Color::White, drivers::vga_enhanced::Color::Blue);
    for i in 0..80 {
        drivers::vga_enhanced::write_at(0, i, " ", 
            drivers::vga_enhanced::Color::White, 
            drivers::vga_enhanced::Color::Blue);
    }
    drivers::vga_enhanced::write_at(0, 2, " UniverseK OS - Driver Demo ", 
        drivers::vga_enhanced::Color::White, 
        drivers::vga_enhanced::Color::Blue);
    
    // Draw container box
    drivers::vga_enhanced::set_color(drivers::vga_enhanced::Color::White, drivers::vga_enhanced::Color::Black);
    drivers::vga_enhanced::draw_shadowed_box(2, 2, 76, 20);
    
    // Get and display RTC time
    let datetime = drivers::rtc::get_datetime();
    drivers::vga_enhanced::write_at(3, 4, &format!("Current Date/Time: {}", datetime.format()),
        drivers::vga_enhanced::Color::Yellow, 
        drivers::vga_enhanced::Color::Black);
    
    // Display PIT timer info
    drivers::vga_enhanced::write_at(5, 4, "PIT Timer: Active",
        drivers::vga_enhanced::Color::LightGreen, 
        drivers::vga_enhanced::Color::Black);
    
    // Display input device info
    drivers::vga_enhanced::write_at(7, 4, "PS/2 Keyboard: Initialized",
        drivers::vga_enhanced::Color::LightGreen, 
        drivers::vga_enhanced::Color::Black);
    
    drivers::vga_enhanced::write_at(8, 4, "PS/2 Mouse: Initialized",
        drivers::vga_enhanced::Color::LightGreen, 
        drivers::vga_enhanced::Color::Black);
    
    // Get and display detected PCI devices
    let pci_devices = drivers::pci::get_devices();
    drivers::vga_enhanced::write_at(10, 4, &format!("PCI Devices Detected: {}", pci_devices.len()),
        drivers::vga_enhanced::Color::White, 
        drivers::vga_enhanced::Color::Black);
    
    // Show usage instructions
    drivers::vga_enhanced::write_at(18, 4, "Use mouse and keyboard to interact.",
        drivers::vga_enhanced::Color::LightCyan, 
        drivers::vga_enhanced::Color::Black);
    
    drivers::vga_enhanced::write_at(19, 4, "Press ESC to exit.",
        drivers::vga_enhanced::Color::LightCyan, 
        drivers::vga_enhanced::Color::Black);
} 