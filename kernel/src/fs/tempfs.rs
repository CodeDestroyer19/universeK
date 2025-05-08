use crate::errors::KernelError;
use crate::fs::vfs::{FileSystem, FileHandle, Metadata, DirEntry, NodeType};
use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::format;
use spin::Mutex;
use core::sync::atomic::{AtomicUsize, Ordering};
use crate::serial_println;

/// A simple in-memory file system
pub struct TempFs {
    name: String,
    nodes: BTreeMap<String, TempFsNode>,
    next_inode: AtomicUsize,
}

/// File or directory in the TempFs
pub struct TempFsNode {
    pub inode: usize,
    pub metadata: Metadata,
    pub data: NodeData,
}

/// Content of a node (either file or directory)
pub enum NodeData {
    File(Vec<u8>),
    Directory(BTreeMap<String, usize>), // filename -> inode
}

impl TempFs {
    pub fn new(name: &str) -> Self {
        serial_println!("DEBUG: Creating new TempFs with name '{}'", name);
        
        let mut fs = Self {
            name: name.to_string(),
            nodes: BTreeMap::new(),
            next_inode: AtomicUsize::new(1), // Start at inode 1
        };
        
        // Create root directory
        serial_println!("DEBUG: Creating root directory for TempFs");
        let root_inode = fs.next_inode.fetch_add(1, Ordering::SeqCst);
        let root_dir = TempFsNode {
            inode: root_inode,
            metadata: Metadata::new_directory(),
            data: NodeData::Directory(BTreeMap::new()),
        };
        
        fs.nodes.insert("/".to_string(), root_dir);
        serial_println!("DEBUG: TempFs initialization complete");
        
        fs
    }
    
    /// Normalizes a path (removes trailing slashes, adds leading slash if missing)
    fn normalize_path(&self, path: &str) -> String {
        serial_println!("DEBUG: TempFS::normalize_path - Input path: '{}'", path);
        
        let mut normalized = path.to_string();
        
        // Add leading slash if missing
        if !normalized.starts_with('/') {
            normalized = format!("/{}", normalized);
        }
        
        // Remove trailing slash if present (except for root)
        if normalized.len() > 1 && normalized.ends_with('/') {
            normalized.pop();
        }
        
        serial_println!("DEBUG: TempFS::normalize_path - Normalized: '{}'", normalized);
        normalized
    }
    
    /// Gets the parent directory path and filename from a path
    fn split_path(&self, path: &str) -> (String, String) {
        let normalized = self.normalize_path(path);
        
        if normalized == "/" {
            return ("/".to_string(), "".to_string());
        }
        
        let last_slash = normalized.rfind('/').unwrap();
        let parent = normalized[..=last_slash].to_string();
        let filename = normalized[last_slash+1..].to_string();
        
        // If parent is empty, it's the root
        let parent = if parent.is_empty() { "/".to_string() } else { parent };
        
        (parent, filename)
    }
    
    /// Checks if a node exists
    fn node_exists(&self, path: &str) -> bool {
        let normalized = self.normalize_path(path);
        self.nodes.contains_key(&normalized)
    }
    
    /// Gets a node reference
    fn get_node(&self, path: &str) -> Result<&TempFsNode, KernelError> {
        let normalized = self.normalize_path(path);
        self.nodes.get(&normalized).ok_or(KernelError::NotFound)
    }
    
    /// Gets a mutable node reference
    fn get_node_mut(&mut self, path: &str) -> Result<&mut TempFsNode, KernelError> {
        let normalized = self.normalize_path(path);
        
        serial_println!("DEBUG: TempFS::get_node_mut - Looking up path: '{}'", normalized);
        serial_println!("DEBUG: TempFS::get_node_mut - Bypassed node listing. Attempting get_mut directly.");
        
        match self.nodes.get_mut(&normalized) {
            Some(node) => {
                serial_println!("DEBUG: TempFS::get_node_mut - Found node for path: '{}'", normalized);
                Ok(node)
            },
            None => {
                serial_println!("DEBUG: TempFS::get_node_mut - Node not found for path: '{}'", normalized);
                Err(KernelError::NotFound)
            }
        }
    }
    
    /// Adds a node to a directory
    fn add_to_directory(&mut self, parent_path: &str, filename: &str, inode: usize) -> Result<(), KernelError> {
        let parent = self.get_node_mut(parent_path)?;
        
        // Check that parent is a directory
        if let NodeData::Directory(entries) = &mut parent.data {
            entries.insert(filename.to_string(), inode);
            Ok(())
        } else {
            Err(KernelError::NotADirectory)
        }
    }
}

impl FileSystem for TempFs {
    fn mount(&mut self) -> Result<(), KernelError> {
        serial_println!("DEBUG: Mounting TempFs '{}'", self.name);
        // Nothing to do for an in-memory file system
        Ok(())
    }
    
    fn unmount(&mut self) -> Result<(), KernelError> {
        serial_println!("DEBUG: Unmounting TempFs '{}'", self.name);
        // Nothing to do for an in-memory file system
        Ok(())
    }
    
    fn create_file(&mut self, path: &str) -> Result<(), KernelError> {
        let (parent_path, filename) = self.split_path(path);
        
        // Check if file already exists
        let normalized = self.normalize_path(path);
        if self.node_exists(&normalized) {
            return Err(KernelError::AlreadyExists);
        }
        
        // Check if parent directory exists
        if !self.node_exists(&parent_path) {
            return Err(KernelError::NotFound);
        }
        
        // Create the file
        let inode = self.next_inode.fetch_add(1, Ordering::SeqCst);
        let file_node = TempFsNode {
            inode,
            metadata: Metadata::new_file(),
            data: NodeData::File(Vec::new()),
        };
        
        // Add to file system
        self.nodes.insert(normalized, file_node);
        
        // Add to parent directory
        self.add_to_directory(&parent_path, &filename, inode)?;
        
        Ok(())
    }
    
    fn create_directory(&mut self, path: &str) -> Result<(), KernelError> {
        let (parent_path, dirname) = self.split_path(path);
        
        // Check if directory already exists
        let normalized = self.normalize_path(path);
        if self.node_exists(&normalized) {
            return Err(KernelError::AlreadyExists);
        }
        
        // Check if parent directory exists
        if !self.node_exists(&parent_path) {
            return Err(KernelError::NotFound);
        }
        
        // Create the directory
        let inode = self.next_inode.fetch_add(1, Ordering::SeqCst);
        let dir_node = TempFsNode {
            inode,
            metadata: Metadata::new_directory(),
            data: NodeData::Directory(BTreeMap::new()),
        };
        
        // Add to file system
        self.nodes.insert(normalized, dir_node);
        
        // Add to parent directory
        self.add_to_directory(&parent_path, &dirname, inode)?;
        
        Ok(())
    }
    
    fn remove(&mut self, path: &str) -> Result<(), KernelError> {
        let normalized = self.normalize_path(path);
        
        // Can't remove root
        if normalized == "/" {
            return Err(KernelError::InvalidOperation);
        }
        
        // Check if the node exists
        if !self.node_exists(&normalized) {
            return Err(KernelError::NotFound);
        }
        
        // Check if it's a directory and if it's empty
        if let Some(node) = self.nodes.get(&normalized) {
            if let NodeData::Directory(entries) = &node.data {
                if !entries.is_empty() {
                    return Err(KernelError::DirectoryNotEmpty);
                }
            }
        }
        
        // Remove from parent directory
        let (parent_path, filename) = self.split_path(&normalized);
        let parent = self.get_node_mut(&parent_path)?;
        
        if let NodeData::Directory(entries) = &mut parent.data {
            entries.remove(&filename);
        } else {
            return Err(KernelError::NotADirectory);
        }
        
        // Remove from the file system
        self.nodes.remove(&normalized);
        
        Ok(())
    }
    
    fn open(&self, path: &str, write: bool) -> Result<FileHandle, KernelError> {
        // Check if the file exists
        let normalized = self.normalize_path(path);
        
        serial_println!("DEBUG: TempFS: Opening file '{}'", normalized);
        
        if !self.node_exists(&normalized) {
            serial_println!("DEBUG: TempFS: File not found: {}", normalized);
            return Err(KernelError::NotFound);
        }
        
        // Check if it's a file
        let node = self.get_node(&normalized)?;
        if let NodeData::Directory(_) = node.data {
            serial_println!("DEBUG: TempFS: Cannot open directory as file: {}", normalized);
            return Err(KernelError::IsADirectory);
        }
        
        // Create a file handle with flags
        let flags = if write {
            crate::fs::vfs::file_flags::READ | crate::fs::vfs::file_flags::WRITE
        } else {
            crate::fs::vfs::file_flags::READ
        };
        
        // Get the VFS manager for the filesystem this TempFs is part of
        serial_println!("DEBUG: TempFS: Creating file handle");
        
        // We need to find our own instance in the VFS mount points
        let vfs = crate::fs::vfs::get_vfs_manager().ok_or(KernelError::NotInitialized)?;
        let fs = vfs.find_fs(path)?;
        
        // Create and return the file handle
        let handle = FileHandle::new(&normalized, fs, flags);
        serial_println!("DEBUG: TempFS: File handle created successfully");
        Ok(handle)
    }
    
    fn metadata(&self, path: &str) -> Result<Metadata, KernelError> {
        let normalized = self.normalize_path(path);
        let node = self.get_node(&normalized)?;
        
        Ok(node.metadata.clone())
    }
    
    fn read_dir(&self, path: &str) -> Result<Vec<DirEntry>, KernelError> {
        let normalized = self.normalize_path(path);
        let node = self.get_node(&normalized)?;
        
        if let NodeData::Directory(entries) = &node.data {
            let mut result = Vec::new();
            
            for (name, &inode) in entries {
                // Find the node with this inode
                let node_path = if normalized == "/" {
                    format!("/{}", name)
                } else {
                    format!("{}/{}", normalized, name)
                };
                
                if let Some(node) = self.nodes.get(&node_path) {
                    let node_type = match node.data {
                        NodeData::File(_) => NodeType::File,
                        NodeData::Directory(_) => NodeType::Directory,
                    };
                    
                    result.push(DirEntry::new(name, node_type, inode));
                }
            }
            
            Ok(result)
        } else {
            Err(KernelError::NotADirectory)
        }
    }
    
    fn rename(&mut self, from: &str, to: &str) -> Result<(), KernelError> {
        let from_normalized = self.normalize_path(from);
        let to_normalized = self.normalize_path(to);
        
        // Check if source exists
        if !self.node_exists(&from_normalized) {
            return Err(KernelError::NotFound);
        }
        
        // Check if destination already exists
        if self.node_exists(&to_normalized) {
            return Err(KernelError::AlreadyExists);
        }
        
        // Get the node
        if let Some(node) = self.nodes.remove(&from_normalized) {
            // Add to destination
            self.nodes.insert(to_normalized.clone(), node);
            
            // Update parent directories
            let (from_parent, from_name) = self.split_path(&from_normalized);
            let (to_parent, to_name) = self.split_path(&to_normalized);
            
            // Remove from old parent
            if let Ok(parent) = self.get_node_mut(&from_parent) {
                if let NodeData::Directory(entries) = &mut parent.data {
                    let inode = entries.remove(&from_name);
                    
                    // Add to new parent
                    if let Some(inode) = inode {
                        self.add_to_directory(&to_parent, &to_name, inode)?;
                    }
                }
            }
            
            Ok(())
        } else {
            Err(KernelError::NotFound)
        }
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn total_space(&self) -> u64 {
        // Since this is an in-memory file system, we'll return a fixed size
        1024 * 1024 // 1 MB
    }
    
    fn available_space(&self) -> u64 {
        // Calculate used space
        let mut used = 0u64;
        for node in self.nodes.values() {
            if let NodeData::File(data) = &node.data {
                used += data.len() as u64;
            }
        }
        
        // Return available space
        self.total_space().saturating_sub(used)
    }
    
    fn read_at(&self, path: &str, offset: u64, buffer: &mut [u8]) -> Result<usize, KernelError> {
        serial_println!("DEBUG: TempFS read_at: path={}, offset={}, buffer_len={}", path, offset, buffer.len());
        self.read_file(path, offset, buffer)
    }
    
    fn write_at(&mut self, path: &str, offset: u64, buffer: &[u8]) -> Result<usize, KernelError> {
        serial_println!("DEBUG: TempFS write_at: path={}, offset={}, buffer_len={}", path, offset, buffer.len());
        self.write_file(path, offset, buffer)
    }
}

impl TempFs {
    /// Read data from a file at the given offset
    pub fn read_file(&self, path: &str, offset: u64, buffer: &mut [u8]) -> Result<usize, KernelError> {
        let normalized = self.normalize_path(path);
        let node = self.get_node(&normalized)?;
        
        if let NodeData::File(data) = &node.data {
            let offset = offset as usize;
            if offset >= data.len() {
                return Ok(0); // EOF
            }
            
            let bytes_to_read = core::cmp::min(buffer.len(), data.len() - offset);
            buffer[..bytes_to_read].copy_from_slice(&data[offset..offset + bytes_to_read]);
            
            Ok(bytes_to_read)
        } else {
            Err(KernelError::NotAFile)
        }
    }
    
    /// Write data to a file at the given offset
    pub fn write_file(&mut self, path: &str, offset: u64, buffer: &[u8]) -> Result<usize, KernelError> {
        serial_println!("DEBUG: TempFS::write_file - Starting with path={}, offset={}, buffer.len={}", 
                        path, offset, buffer.len());
        
        let normalized = self.normalize_path(path);
        serial_println!("DEBUG: TempFS::write_file - Normalized path: {}", normalized);
        
        let node = match self.get_node_mut(&normalized) {
            Ok(n) => {
                serial_println!("DEBUG: TempFS::write_file - Found node");
                n
            },
            Err(e) => {
                serial_println!("DEBUG: TempFS::write_file - Node not found: {:?}", e);
                return Err(e);
            }
        };
        
        if let NodeData::File(data) = &mut node.data {
            let offset = offset as usize;
            serial_println!("DEBUG: TempFS::write_file - File node found, current size={}, writing at offset={}", 
                            data.len(), offset);
            
            // Resize the file if necessary
            if offset + buffer.len() > data.len() {
                serial_println!("DEBUG: TempFS::write_file - Resizing file from {} to {} bytes", 
                                data.len(), offset + buffer.len());
                data.resize(offset + buffer.len(), 0);
            }
            
            // Write the data
            serial_println!("DEBUG: TempFS::write_file - Copying {} bytes of data", buffer.len());
            data[offset..offset + buffer.len()].copy_from_slice(buffer);
            
            // Update metadata
            node.metadata.size = data.len() as u64;
            node.metadata.modified_at = 0; // Would use current time in a real implementation
            
            serial_println!("DEBUG: TempFS::write_file - Write successful, new file size={}", data.len());
            Ok(buffer.len())
        } else {
            serial_println!("DEBUG: TempFS::write_file - Not a file");
            Err(KernelError::NotAFile)
        }
    }
} 