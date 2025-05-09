use crate::errors::KernelError;
use crate::fs::vfs::{FileSystem, FileHandle, Metadata, DirEntry, NodeType};
use crate::fs::block_device::BlockDevice;
use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use alloc::sync::Arc;
use spin::Mutex;
use core::mem::size_of;

// FAT filesystem types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FatType {
    Fat12,
    Fat16,
    Fat32,
}

// FAT specific errors
#[derive(Debug)]
pub enum FatError {
    InvalidSignature,
    UnsupportedFat,
    InvalidParameter,
    ReadError,
    WriteError,
    NotFound,
    AlreadyExists,
    DirectoryFull,
    NotADirectory,
    NotAFile,
    IoError,
}

impl From<FatError> for KernelError {
    fn from(err: FatError) -> Self {
        match err {
            FatError::InvalidSignature => KernelError::InvalidData,
            FatError::UnsupportedFat => KernelError::UnsupportedFeature,
            FatError::InvalidParameter => KernelError::InvalidParameter,
            FatError::ReadError => KernelError::ReadError,
            FatError::WriteError => KernelError::WriteError,
            FatError::NotFound => KernelError::NotFound,
            FatError::AlreadyExists => KernelError::AlreadyExists,
            FatError::DirectoryFull => KernelError::DirectoryFull,
            FatError::NotADirectory => KernelError::NotADirectory,
            FatError::NotAFile => KernelError::NotAFile,
            FatError::IoError => KernelError::IoError,
        }
    }
}

// FAT Boot Parameter Block (BPB) for FAT16/FAT32
#[repr(packed)]
pub struct FatBootSector {
    jmp_boot: [u8; 3],
    oem_name: [u8; 8],
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors: u16,
    fat_count: u8,
    root_entry_count: u16,
    total_sectors_16: u16,
    media_type: u8,
    sectors_per_fat_16: u16,
    sectors_per_track: u16,
    head_count: u16,
    hidden_sectors: u32,
    total_sectors_32: u32,
    // FAT32 specific fields
    sectors_per_fat_32: u32,
    extended_flags: u16,
    fs_version: u16,
    root_cluster: u32,
    fs_info_sector: u16,
    backup_boot_sector: u16,
    reserved: [u8; 12],
    drive_number: u8,
    reserved1: u8,
    boot_signature: u8,
    volume_id: u32,
    volume_label: [u8; 11],
    fs_type: [u8; 8],
}

// FAT directory entry format
#[derive(Debug, Clone, Copy)]
#[repr(packed)]
pub struct FatDirEntry {
    name: [u8; 8],
    ext: [u8; 3],
    attr: u8,
    reserved: u8,
    create_time_tenth: u8,
    create_time: u16,
    create_date: u16,
    access_date: u16,
    cluster_high: u16,
    modify_time: u16,
    modify_date: u16,
    cluster_low: u16,
    size: u32,
}

// Attribute bits for FAT directory entries
const ATTR_READ_ONLY: u8 = 0x01;
const ATTR_HIDDEN: u8 = 0x02;
const ATTR_SYSTEM: u8 = 0x04;
const ATTR_VOLUME_ID: u8 = 0x08;
const ATTR_DIRECTORY: u8 = 0x10;
const ATTR_ARCHIVE: u8 = 0x20;
const ATTR_LONG_NAME: u8 = ATTR_READ_ONLY | ATTR_HIDDEN | ATTR_SYSTEM | ATTR_VOLUME_ID;

// Special FAT cluster values
const FAT_EOC: u32 = 0x0FFFFFF8; // End of cluster chain
const FAT_BAD: u32 = 0x0FFFFFF7; // Bad cluster

// The FAT file system implementation
pub struct FatFileSystem {
    // The underlying block device
    device: Arc<Mutex<dyn BlockDevice>>,
    // FAT type
    fat_type: FatType,
    // Boot sector data
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    sectors_per_fat: u32,
    fat_count: u8,
    reserved_sectors: u16,
    root_entry_count: u16,
    root_directory_sectors: u32,
    first_data_sector: u32,
    first_fat_sector: u32,
    data_sectors: u32,
    total_clusters: u32,
    // For FAT32
    root_cluster: u32,
}

impl FatFileSystem {
    // Create a new FAT file system from a block device
    pub fn new(device: Arc<Mutex<dyn BlockDevice>>) -> Result<Self, KernelError> {
        let mut fs = Self {
            device: device.clone(),
            fat_type: FatType::Fat16, // Will be determined later
            bytes_per_sector: 0,
            sectors_per_cluster: 0,
            sectors_per_fat: 0,
            fat_count: 0,
            reserved_sectors: 0,
            root_entry_count: 0,
            root_directory_sectors: 0,
            first_data_sector: 0,
            first_fat_sector: 0,
            data_sectors: 0,
            total_clusters: 0,
            root_cluster: 0,
        };
        
        fs.read_boot_sector()?;
        Ok(fs)
    }
    
    // Read and parse the boot sector
    fn read_boot_sector(&mut self) -> Result<(), KernelError> {
        // Read the boot sector (first sector of the volume)
        let mut buffer = vec![0u8; 512];
        {
            let device = self.device.lock();
            device.read_block(0, &mut buffer).map_err(|_| FatError::ReadError)?;
        }
        
        // Parse the boot sector
        let boot_sector: &FatBootSector = unsafe {
            &*(buffer.as_ptr() as *const FatBootSector)
        };
        
        // Check the signature
        if boot_sector.boot_signature != 0x29 {
            return Err(FatError::InvalidSignature.into());
        }
        
        // Get basic FAT parameters
        self.bytes_per_sector = boot_sector.bytes_per_sector;
        self.sectors_per_cluster = boot_sector.sectors_per_cluster;
        self.reserved_sectors = boot_sector.reserved_sectors;
        self.fat_count = boot_sector.fat_count;
        self.root_entry_count = boot_sector.root_entry_count;
        
        // Determine the FAT type and specific parameters
        let total_sectors = if boot_sector.total_sectors_16 != 0 {
            boot_sector.total_sectors_16 as u32
        } else {
            boot_sector.total_sectors_32
        };
        
        self.sectors_per_fat = if boot_sector.sectors_per_fat_16 != 0 {
            boot_sector.sectors_per_fat_16 as u32
        } else {
            boot_sector.sectors_per_fat_32
        };
        
        // Calculate derived values
        self.root_directory_sectors = ((self.root_entry_count as u32 * 32) + (self.bytes_per_sector as u32 - 1)) / (self.bytes_per_sector as u32);
        self.first_fat_sector = self.reserved_sectors as u32;
        self.first_data_sector = self.reserved_sectors as u32 + 
                                (self.fat_count as u32 * self.sectors_per_fat) + 
                                self.root_directory_sectors;
        
        self.data_sectors = total_sectors - self.first_data_sector;
        self.total_clusters = self.data_sectors / (self.sectors_per_cluster as u32);
        
        // Determine the FAT type based on cluster count
        self.fat_type = if self.total_clusters < 4085 {
            FatType::Fat12
        } else if self.total_clusters < 65525 {
            FatType::Fat16
        } else {
            FatType::Fat32
        };
        
        // For FAT32, get the root cluster
        if self.fat_type == FatType::Fat32 {
            self.root_cluster = boot_sector.root_cluster;
        }
        
        Ok(())
    }
    
    // Convert a cluster number to a sector number
    fn cluster_to_sector(&self, cluster: u32) -> u32 {
        self.first_data_sector + ((cluster - 2) * self.sectors_per_cluster as u32)
    }
    
    // Read an entry from the FAT
    fn read_fat_entry(&self, cluster: u32) -> Result<u32, KernelError> {
        let fat_offset = match self.fat_type {
            FatType::Fat12 => cluster * 3 / 2,
            FatType::Fat16 => cluster * 2,
            FatType::Fat32 => cluster * 4,
        };
        
        let fat_sector = self.first_fat_sector + (fat_offset / self.bytes_per_sector as u32);
        let entry_offset = (fat_offset % self.bytes_per_sector as u32) as usize;
        
        let mut buffer = vec![0u8; self.bytes_per_sector as usize];
        {
            let device = self.device.lock();
            device.read_block(fat_sector as u64, &mut buffer).map_err(|_| FatError::ReadError)?;
        }
        
        match self.fat_type {
            FatType::Fat12 => {
                let mut value = if entry_offset < (self.bytes_per_sector as usize) - 1 {
                    ((buffer[entry_offset + 1] as u32) << 8) | (buffer[entry_offset] as u32)
                } else {
                    // Entry spans two sectors, read the next sector
                    let mut next_buffer = vec![0u8; self.bytes_per_sector as usize];
                    {
                        let device = self.device.lock();
                        device.read_block((fat_sector + 1) as u64, &mut next_buffer)
                            .map_err(|_| FatError::ReadError)?;
                    }
                    ((next_buffer[0] as u32) << 8) | (buffer[entry_offset] as u32)
                };
                
                // Get correct 12-bit value based on even/odd cluster number
                if cluster & 1 == 0 {
                    // Even: take low 12 bits
                    value &= 0xFFF;
                } else {
                    // Odd: take high 12 bits
                    value >>= 4;
                }
                
                Ok(value)
            },
            FatType::Fat16 => {
                let value = ((buffer[entry_offset + 1] as u32) << 8) | (buffer[entry_offset] as u32);
                Ok(value)
            },
            FatType::Fat32 => {
                let value = ((buffer[entry_offset + 3] as u32) << 24) |
                           ((buffer[entry_offset + 2] as u32) << 16) |
                           ((buffer[entry_offset + 1] as u32) << 8) |
                            (buffer[entry_offset] as u32);
                // Only lower 28 bits are used
                Ok(value & 0x0FFFFFFF)
            },
        }
    }
    
    // Read a cluster into a buffer
    fn read_cluster(&self, cluster: u32, buffer: &mut [u8]) -> Result<(), KernelError> {
        let first_sector = self.cluster_to_sector(cluster);
        let bytes_per_cluster = self.sectors_per_cluster as usize * self.bytes_per_sector as usize;
        
        if buffer.len() < bytes_per_cluster {
            return Err(FatError::InvalidParameter.into());
        }
        
        // Read all sectors in the cluster
        for i in 0..self.sectors_per_cluster {
            let sector = first_sector + i as u32;
            let offset = i as usize * self.bytes_per_sector as usize;
            let end = offset + self.bytes_per_sector as usize;
            
            let device = self.device.lock();
            device.read_block(sector as u64, &mut buffer[offset..end])
                .map_err(|_| FatError::ReadError)?;
        }
        
        Ok(())
    }
    
    // Read a file cluster chain into a buffer
    fn read_file(&self, start_cluster: u32, buffer: &mut [u8]) -> Result<usize, KernelError> {
        let mut total_read = 0;
        let bytes_per_cluster = self.sectors_per_cluster as usize * self.bytes_per_sector as usize;
        let mut cluster = start_cluster;
        
        while cluster < FAT_EOC && total_read < buffer.len() {
            let bytes_to_read = core::cmp::min(bytes_per_cluster, buffer.len() - total_read);
            
            // Allocate a temporary buffer for the cluster
            let mut cluster_buffer = vec![0u8; bytes_per_cluster];
            self.read_cluster(cluster, &mut cluster_buffer)?;
            
            // Copy data to the output buffer
            buffer[total_read..total_read + bytes_to_read]
                .copy_from_slice(&cluster_buffer[..bytes_to_read]);
            
            total_read += bytes_to_read;
            
            // Get next cluster in the chain
            if total_read < buffer.len() {
                let next_cluster = self.read_fat_entry(cluster)?;
                if next_cluster >= FAT_EOC {
                    break; // End of file
                }
                cluster = next_cluster;
            } else {
                break; // Buffer is full
            }
        }
        
        Ok(total_read)
    }
    
    // Read the root directory (FAT12/16)
    fn read_root_directory(&self) -> Result<Vec<FatDirEntry>, KernelError> {
        if self.fat_type == FatType::Fat32 {
            // For FAT32, root directory is a cluster chain starting at root_cluster
            return self.read_directory(self.root_cluster);
        }
        
        // For FAT12/16, root directory is at a fixed location
        let root_dir_size = self.root_entry_count as usize * size_of::<FatDirEntry>();
        let mut buffer = vec![0u8; root_dir_size];
        
        let mut offset = 0;
        for i in 0..self.root_directory_sectors {
            let sector = self.reserved_sectors as u32 + (self.fat_count as u32 * self.sectors_per_fat) + i;
            let sector_size = self.bytes_per_sector as usize;
            
            if offset + sector_size > buffer.len() {
                break;
            }
            
            let device = self.device.lock();
            device.read_block(sector as u64, &mut buffer[offset..offset + sector_size])
                .map_err(|_| FatError::ReadError)?;
            
            offset += sector_size;
        }
        
        // Parse the directory entries
        let mut entries = Vec::new();
        let mut i = 0;
        while i + size_of::<FatDirEntry>() <= buffer.len() {
            let entry_ptr = &buffer[i] as *const u8 as *const FatDirEntry;
            let entry = unsafe { *entry_ptr };
            
            // Check if this is a valid entry
            if entry.name[0] != 0 && entry.name[0] != 0xE5 {
                entries.push(entry);
            } else if entry.name[0] == 0 {
                break; // End of directory
            }
            
            i += size_of::<FatDirEntry>();
        }
        
        Ok(entries)
    }
    
    // Read a directory (non-root for FAT12/16, any for FAT32)
    fn read_directory(&self, cluster: u32) -> Result<Vec<FatDirEntry>, KernelError> {
        let mut entries = Vec::new();
        let mut current_cluster = cluster;
        
        while current_cluster < FAT_EOC {
            let bytes_per_cluster = self.sectors_per_cluster as usize * self.bytes_per_sector as usize;
            let mut buffer = vec![0u8; bytes_per_cluster];
            
            self.read_cluster(current_cluster, &mut buffer)?;
            
            // Parse the directory entries in this cluster
            let mut i = 0;
            while i + size_of::<FatDirEntry>() <= buffer.len() {
                let entry_ptr = &buffer[i] as *const u8 as *const FatDirEntry;
                let entry = unsafe { *entry_ptr };
                
                // Check if this is a valid entry
                if entry.name[0] != 0 && entry.name[0] != 0xE5 {
                    entries.push(entry);
                } else if entry.name[0] == 0 {
                    break; // End of directory
                }
                
                i += size_of::<FatDirEntry>();
            }
            
            // Go to the next cluster in the chain
            let next_cluster = self.read_fat_entry(current_cluster)?;
            if next_cluster >= FAT_EOC {
                break;
            }
            current_cluster = next_cluster;
        }
        
        Ok(entries)
    }
    
    // Convert an 8.3 filename to a string
    fn fat_name_to_string(&self, name: &[u8; 8], ext: &[u8; 3]) -> String {
        let mut result = String::new();
        
        // Add the base name (trim trailing spaces)
        let base_len = name.iter().position(|&c| c == b' ').unwrap_or(8);
        for i in 0..base_len {
            result.push(name[i] as char);
        }
        
        // Add the extension if it's not empty
        let ext_len = ext.iter().position(|&c| c == b' ').unwrap_or(3);
        if ext_len > 0 {
            result.push('.');
            for i in 0..ext_len {
                result.push(ext[i] as char);
            }
        }
        
        result
    }
    
    // Find a file or directory in a directory
    fn find_in_directory(&self, dir_entries: &[FatDirEntry], name: &str) -> Option<FatDirEntry> {
        // Convert name to uppercase 8.3 format
        let name_upper = name.to_uppercase();
        
        // Split into base and extension
        let (_base, _ext) = if let Some(dot_pos) = name_upper.find('.') {
            (&name_upper[..dot_pos], &name_upper[dot_pos + 1..])
        } else {
            (&name_upper[..], "")
        };
        
        // Find the entry
        for entry in dir_entries {
            let entry_name = self.fat_name_to_string(&entry.name, &entry.ext);
            if entry_name.to_uppercase() == name_upper {
                // Clone the entry instead of dereferencing it
                return Some(FatDirEntry {
                    name: entry.name,
                    ext: entry.ext,
                    attr: entry.attr,
                    reserved: entry.reserved,
                    create_time_tenth: entry.create_time_tenth,
                    create_time: entry.create_time,
                    create_date: entry.create_date,
                    access_date: entry.access_date,
                    cluster_high: entry.cluster_high,
                    modify_time: entry.modify_time,
                    modify_date: entry.modify_date,
                    cluster_low: entry.cluster_low,
                    size: entry.size,
                });
            }
        }
        
        None
    }
    
    // Get the starting cluster of a file/directory
    fn get_cluster(entry: &FatDirEntry) -> u32 {
        let cluster_high = (entry.cluster_high as u32) << 16;
        let cluster_low = entry.cluster_low as u32;
        cluster_high | cluster_low
    }
    
    // Check if an entry is a directory
    fn is_directory(entry: &FatDirEntry) -> bool {
        entry.attr & ATTR_DIRECTORY != 0
    }
    
    // Convert a path to a FatDirEntry
    fn path_to_entry(&self, path: &str) -> Result<FatDirEntry, KernelError> {
        // Normalize the path
        let path = if path.starts_with('/') {
            &path[1..]
        } else {
            path
        };
        
        // Empty path is the root directory
        if path.is_empty() {
            // Create a synthetic entry for the root directory
            let mut root_entry = FatDirEntry {
                name: [b' '; 8],
                ext: [b' '; 3],
                attr: ATTR_DIRECTORY,
                reserved: 0,
                create_time_tenth: 0,
                create_time: 0,
                create_date: 0,
                access_date: 0,
                cluster_high: 0,
                modify_time: 0,
                modify_date: 0,
                cluster_low: 0,
                size: 0,
            };
            
            // For FAT32, root directory starts at root_cluster
            if self.fat_type == FatType::Fat32 {
                root_entry.cluster_low = self.root_cluster as u16;
            }
            
            return Ok(root_entry);
        }
        
        // Split the path into components
        let components: Vec<&str> = path.split('/').collect();
        
        // Start at the root directory
        let mut current_entries = self.read_root_directory()?;
        
        // Traverse the path
        for (i, component) in components.iter().enumerate() {
            // Find the component in the current directory
            let entry = self.find_in_directory(&current_entries, component)
                .ok_or(FatError::NotFound)?;
            
            if i == components.len() - 1 {
                // Last component, return the entry
                return Ok(entry);
            } else {
                // Not the last component, must be a directory
                if !Self::is_directory(&entry) {
                    return Err(FatError::NotADirectory.into());
                }
                
                // Read the next directory
                let cluster = Self::get_cluster(&entry);
                current_entries = self.read_directory(cluster)?;
            }
        }
        
        // This shouldn't happen
        Err(FatError::NotFound.into())
    }
}

impl FileSystem for FatFileSystem {
    fn mount(&mut self) -> Result<(), KernelError> {
        // Already mounted when created
        Ok(())
    }
    
    fn unmount(&mut self) -> Result<(), KernelError> {
        // Nothing to do
        Ok(())
    }
    
    fn create_file(&mut self, _path: &str) -> Result<(), KernelError> {
        // Not implemented yet
        Err(KernelError::NotImplemented)
    }
    
    fn create_directory(&mut self, _path: &str) -> Result<(), KernelError> {
        // Not implemented yet
        Err(KernelError::NotImplemented)
    }
    
    fn remove(&mut self, _path: &str) -> Result<(), KernelError> {
        // Not implemented yet
        Err(KernelError::NotImplemented)
    }
    
    fn open(&self, _path: &str, _write: bool) -> Result<FileHandle, KernelError> {
        // Not implemented yet
        Err(KernelError::NotImplemented)
    }
    
    fn metadata(&self, path: &str) -> Result<Metadata, KernelError> {
        // Find the entry for the path
        let entry = self.path_to_entry(path)?;
        
        // Create metadata from the entry
        let node_type = if Self::is_directory(&entry) {
            NodeType::Directory
        } else {
            NodeType::File
        };
        
        let mut metadata = Metadata {
            node_type,
            size: entry.size as u64,
            permissions: 0,
            created_at: 0,
            modified_at: 0,
            accessed_at: 0,
        };
        
        // Set permissions based on attributes
        if entry.attr & ATTR_READ_ONLY != 0 {
            // No write permission
            metadata.permissions = 0b0001_0001;
        } else {
            // Read and write
            metadata.permissions = 0b0011_0011;
        }
        
        Ok(metadata)
    }
    
    fn read_dir(&self, path: &str) -> Result<Vec<DirEntry>, KernelError> {
        // Find the entry for the path
        let entry = self.path_to_entry(path)?;
        
        // Make sure it's a directory
        if !Self::is_directory(&entry) {
            return Err(FatError::NotADirectory.into());
        }
        
        // Read the directory entries
        let dir_entries = if path == "/" && self.fat_type != FatType::Fat32 {
            self.read_root_directory()?
        } else {
            let cluster = Self::get_cluster(&entry);
            self.read_directory(cluster)?
        };
        
        // Convert to VFS directory entries
        let mut result = Vec::new();
        for entry in dir_entries {
            // Skip special entries like . and ..
            if entry.name[0] == b'.' {
                continue;
            }
            
            // Skip volume labels
            if entry.attr & ATTR_VOLUME_ID != 0 {
                continue;
            }
            
            // Convert to a string
            let name = self.fat_name_to_string(&entry.name, &entry.ext);
            
            // Create a directory entry
            let node_type = if Self::is_directory(&entry) {
                NodeType::Directory
            } else {
                NodeType::File
            };
            
            result.push(DirEntry::new(&name, node_type, 0));
        }
        
        Ok(result)
    }
    
    fn rename(&mut self, _from: &str, _to: &str) -> Result<(), KernelError> {
        // Not implemented yet
        Err(KernelError::NotImplemented)
    }
    
    fn name(&self) -> &str {
        match self.fat_type {
            FatType::Fat12 => "FAT12",
            FatType::Fat16 => "FAT16",
            FatType::Fat32 => "FAT32",
        }
    }
    
    fn total_space(&self) -> u64 {
        (self.total_clusters as u64) * (self.sectors_per_cluster as u64) * (self.bytes_per_sector as u64)
    }
    
    fn available_space(&self) -> u64 {
        // This would require scanning the FAT to count free clusters
        // Just return a percentage of total space for now
        self.total_space() / 2
    }
} 