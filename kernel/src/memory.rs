// kernel/src/memory.rs
use x86_64::{
    structures::paging::{PageTable, PhysFrame, Size4KiB, FrameAllocator, OffsetPageTable},
    VirtAddr,
    PhysAddr,
};
use bootloader::bootinfo::{MemoryRegion, MemoryRegionType};
use crate::serial_println;

/// Initialize a new OffsetPageTable.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
pub unsafe fn init_page_table(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    serial_println!("DEBUG: memory: Reading level 4 page table");
    let level_4_table_frame = active_level_4_table(physical_memory_offset);
    serial_println!("DEBUG: memory: Creating OffsetPageTable");
    
    let _phys_to_virt = |frame: PhysFrame| -> *mut PageTable {
        let phys = frame.start_address().as_u64();
        let virt = VirtAddr::new(phys + physical_memory_offset.as_u64());
        virt.as_mut_ptr()
    };
    
    let page_table = OffsetPageTable::new(level_4_table_frame, physical_memory_offset);
    serial_println!("DEBUG: memory: OffsetPageTable created successfully");
    page_table
}

/// Returns a mutable reference to the active level 4 table frame.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr)
    -> &'static mut PageTable
{
    use x86_64::registers::control::Cr3;

    serial_println!("DEBUG: memory: Reading CR3 register");
    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    serial_println!("DEBUG: memory: L4 table at physical address: {:?}", phys);
    
    let virt = physical_memory_offset + phys.as_u64();
    serial_println!("DEBUG: memory: L4 table mapped to virtual address: {:?}", virt);
    
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
    serial_println!("DEBUG: memory: L4 table pointer created");

    &mut *page_table_ptr // unsafe
}

/// A FrameAllocator that returns usable frames from the bootloader's memory map.
pub struct BootInfoFrameAllocator {
    memory_map: &'static [MemoryRegion],
    next: usize,
}

impl BootInfoFrameAllocator {
    /// Create a FrameAllocator from the passed memory map.
    ///
    /// This function is unsafe because the caller must guarantee that the passed
    /// memory map is valid. The main requirement is that all frames that are marked
    /// as `USABLE` in it are really unused.
    pub unsafe fn init(memory_map: &'static [MemoryRegion]) -> Self {
        serial_println!("DEBUG: memory: Initializing BootInfoFrameAllocator");
        serial_println!("DEBUG: memory: Memory map contains {} regions", memory_map.len());
        
        // Count usable regions for debugging
        let usable_count = memory_map.iter()
            .filter(|r| r.region_type == MemoryRegionType::Usable)
            .count();
        serial_println!("DEBUG: memory: Found {} usable memory regions", usable_count);
        
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    /// Returns an iterator over the usable frames according to the memory map.
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // get usable regions from memory map
        let regions = self.memory_map.iter();
        let usable_regions = regions
            .filter(|r| r.region_type == MemoryRegionType::Usable);
        // map each region to its address range
        let addr_ranges = usable_regions
            .map(|r| r.range.start_addr()..r.range.end_addr());
        // transform to an iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        // create `PhysFrame` types from the start addresses
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        if let Some(f) = frame {
            serial_println!("DEBUG: memory: Allocated frame at physical address: {:?}", f.start_address());
        } else {
            serial_println!("DEBUG: memory: Failed to allocate frame #{}", self.next);
        }
        self.next += 1;
        frame
    }
}

// TODO: Function to initialize a static allocator instance.
// pub fn init_frame_allocator(boot_info: &'static BootInfo) { ... } 