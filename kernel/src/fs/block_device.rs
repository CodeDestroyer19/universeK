/// A marker trait for BlockDevice, ensures Send+Sync for all block devices
pub trait BlockDeviceMarker: Send + Sync {}

/// Interface for block devices
pub trait BlockDevice: BlockDeviceMarker {
    /// Return the block size in bytes
    fn block_size(&self) -> usize;
    
    /// Return the total number of blocks
    fn block_count(&self) -> u64;
    
    /// Read a block into the provided buffer
    fn read_block(&self, block_id: u64, buffer: &mut [u8]) -> Result<(), &'static str>;
    
    /// Write a block from the provided buffer
    fn write_block(&mut self, block_id: u64, buffer: &[u8]) -> Result<(), &'static str>;

    // It might be useful to have read/write methods that operate on multiple blocks
    // or at byte offsets, but for now, single block operations are sufficient.
}

// We can also define a helper for block size, e.g., 512 bytes, if it's common.
pub const DEFAULT_BLOCK_SIZE: usize = 512; 