use super::block_device::{BlockDevice, DEFAULT_BLOCK_SIZE};
use alloc::vec::Vec;
use crate::serial_println;

// Further reduce the size to minimize memory usage
const RAMDISK_SIZE_BYTES: usize = 2 * 1024; // 2 KiB RamDisk (even smaller)
const BLOCKS_PER_RAMDISK: usize = RAMDISK_SIZE_BYTES / DEFAULT_BLOCK_SIZE;

pub struct RamDisk {
    data: Vec<u8>, // Change to Vec<u8> instead of Box<[u8]> for simpler handling
    block_size: usize,
}

impl RamDisk {
    /// Creates a new RamDisk with a default size and block size.
    pub fn new() -> Result<Self, &'static str> {
        serial_println!("DEBUG: Creating RamDisk with size {} bytes ({} blocks of {} bytes each)", 
            RAMDISK_SIZE_BYTES, BLOCKS_PER_RAMDISK, DEFAULT_BLOCK_SIZE);
        Self::with_size(RAMDISK_SIZE_BYTES, DEFAULT_BLOCK_SIZE)
    }

    /// Creates a new RamDisk with a specified total size and block size.
    pub fn with_size(total_size_bytes: usize, block_size: usize) -> Result<Self, &'static str> {
        serial_println!("DEBUG: RamDisk::with_size called with {} bytes", total_size_bytes);
        
        if total_size_bytes == 0 || block_size == 0 {
            serial_println!("DEBUG: RamDisk error: sizes must be non-zero");
            return Err("Total size and block size must be non-zero.");
        }
        if total_size_bytes % block_size != 0 {
            serial_println!("DEBUG: RamDisk error: total size must be multiple of block size");
            return Err("Total size must be a multiple of block size.");
        }
        
        // Use simpler allocation approach
        serial_println!("DEBUG: Allocating RamDisk memory ({} bytes)", total_size_bytes);
        
        // Create an empty Vec with enough capacity for the first few blocks
        let initial_capacity = core::cmp::min(total_size_bytes, 1024);
        serial_println!("DEBUG: Creating vector with initial capacity of {} bytes", initial_capacity);
        
        // Create the Vec with initial capacity
        let mut data_vec = Vec::with_capacity(initial_capacity);
        serial_println!("DEBUG: Initial Vec created successfully");
        
        // Grow the Vec in smaller chunks to avoid large allocations
        let chunk_size = block_size;
        let num_chunks = total_size_bytes / chunk_size;
        
        serial_println!("DEBUG: Growing Vec in {} chunks of {} bytes each", num_chunks, chunk_size);
        
        for i in 0..num_chunks {
            if i < 3 || i == num_chunks - 1 {
                serial_println!("DEBUG: Adding chunk {}/{}", i+1, num_chunks);
            }
            
            let current_size = data_vec.len();
            // Add a single block worth of zeros
            for _ in 0..chunk_size {
                data_vec.push(0);
            }
            
            if i < 3 || i == num_chunks - 1 {
                serial_println!("DEBUG: Vec size now {} bytes", data_vec.len());
            }
        }
        
        serial_println!("DEBUG: Final Vec size: {} bytes", data_vec.len());
        serial_println!("DEBUG: RamDisk allocation completed successfully");

        Ok(RamDisk {
            data: data_vec, // Keep as Vec instead of converting to Box<[u8]>
            block_size,
        })
    }
}

impl BlockDevice for RamDisk {
    fn block_size(&self) -> usize {
        self.block_size
    }

    fn block_count(&self) -> u64 {
        (self.data.len() / self.block_size) as u64
    }

    fn read_block(&self, block_id: u64, buffer: &mut [u8]) -> Result<(), &'static str> {
        serial_println!("DEBUG: RamDisk: Reading block {}", block_id);
        
        if buffer.len() != self.block_size {
            return Err("Buffer length does not match block size.");
        }
        let num_blocks = self.block_count();
        if block_id >= num_blocks {
            return Err("Block ID out of bounds.");
        }

        let start = (block_id as usize) * self.block_size;
        let end = start + self.block_size;
        
        buffer.copy_from_slice(&self.data[start..end]);
        Ok(())
    }

    fn write_block(&mut self, block_id: u64, buffer: &[u8]) -> Result<(), &'static str> {
        serial_println!("DEBUG: RamDisk: Writing block {}", block_id);
        
        if buffer.len() != self.block_size {
            return Err("Buffer length does not match block size.");
        }
        let num_blocks = self.block_count();
        if block_id >= num_blocks {
            return Err("Block ID out of bounds.");
        }

        let start = (block_id as usize) * self.block_size;
        let end = start + self.block_size;
        
        self.data[start..end].copy_from_slice(buffer);
        Ok(())
    }
}

impl crate::fs::block_device::BlockDeviceMarker for RamDisk {} 