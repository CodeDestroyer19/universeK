// kernel/src/allocator.rs
use linked_list_allocator::LockedHeap;
use x86_64::{
    structures::paging::{mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB},
    VirtAddr,
};
use crate::serial_println;

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 1024 * 1024; // 1 MiB for the initial kernel heap

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Flag to track heap initialization state
static mut HEAP_INITIALIZED: bool = false;

/// Updates the initialization flag
fn set_heap_initialized() {
    unsafe {
        HEAP_INITIALIZED = true;
    }
}

/// Checks if the heap has been initialized
pub fn is_heap_initialized() -> bool {
    unsafe {
        HEAP_INITIALIZED
    }
}

/// Initializes the kernel heap.
///
/// # Arguments
/// * `mapper`: A mutable reference to the `OffsetPageTable` used for mapping.
/// * `frame_allocator`: A mutable reference to the `FrameAllocator` used for allocating frames.
///
/// # Returns
/// `Ok(())` if initialization is successful.
/// `Err(MapToError<Size4KiB>)` if mapping fails.
pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    serial_println!("DEBUG: allocator: Starting heap initialization");
    serial_println!("DEBUG: allocator: Heap will be at 0x{:x} with size {} bytes", HEAP_START, HEAP_SIZE);
    
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    // Count number of pages for debugging
    let page_count = page_range.clone().count();
    serial_println!("DEBUG: allocator: Need to map {} pages for the heap", page_count);

    for (i, page) in page_range.enumerate() {
        serial_println!("DEBUG: allocator: Mapping page {}/{} at {:?}", i+1, page_count, page);
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        serial_println!("DEBUG: allocator: Got frame at physical address {:?}", frame.start_address());
        
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            serial_println!("DEBUG: allocator: Mapping page to frame with flags {:?}", flags);
            mapper.map_to(page, frame, flags, frame_allocator)?.flush();
        }
    }

    // Initialize the allocator with the mapped heap region
    serial_println!("DEBUG: allocator: All pages mapped successfully, initializing heap allocator");
    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }
    serial_println!("DEBUG: allocator: Heap allocator initialized successfully");

    // Set the initialized flag
    set_heap_initialized();

    Ok(())
} 