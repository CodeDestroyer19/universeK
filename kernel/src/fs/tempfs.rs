use alloc::borrow::ToOwned;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicUsize, Ordering};
use spin::Mutex;

use crate::{errors::KernelError, serial_println};
use crate::fs::vfs::{DirEntry, FileHandle, FileSystem, Metadata, NodeType};

/// In-memory file system for temporary storage
pub struct TempFs {
    /// Name of the file system
    name: String,
    /// Root node inode number
    root_inode: usize,
    /// Next available inode number
    next_inode: AtomicUsize,
    /// Map of paths to nodes
    nodes: BTreeMap<String, TempFsNode>,
}

/// Node data variants
enum NodeData {
    File(Vec<u8>),
    Directory(BTreeMap<String, usize>), // Map of name to inode
}

/// File system node
struct TempFsNode {
    /// Inode number
    inode: usize,
    /// Metadata
    metadata: Metadata,
    /// Actual data
    data: NodeData,
}

impl TempFs {
    /// Create a new TempFS
    pub fn new(name: &str) -> Self {
        serial_println!("DEBUG: Creating new TempFS with name: {}", name);
        
        // Create root node
        let root_inode = 1;
        let root_node = TempFsNode {
            inode: root_inode,
            metadata: Metadata::new_directory(),
            data: NodeData::Directory(BTreeMap::new()),
        };
        
        // Create nodes map with root
        let mut nodes = BTreeMap::new();
        nodes.insert("/".to_string(), root_node);
        
        Self {
            name: name.to_string(),
            root_inode,
            next_inode: AtomicUsize::new(2), // Start at 2 because 1 is root
            nodes,
        }
    }

    /// Normalize a path into a canonical form (absolute, no trailing slash, no double slashes)
    pub fn normalize_path_canonical(&self, path: &str) -> String {
        serial_println!("DEBUG: TempFS::normalize_path_canonical - Input: '{}'", path);
        
        // Handle empty path
        if path.is_empty() {
            return "/".to_string();
        }
        
        // Ensure leading slash
        let mut result = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{}", path)
        };
        
        // Remove trailing slashes (except for root)
        while result.len() > 1 && result.ends_with('/') {
            result.pop();
        }
        
        // Fix double slashes
        while result.contains("//") {
            result = result.replace("//", "/");
        }
        
        serial_println!("DEBUG: TempFS::normalize_path_canonical - Output: '{}'", result);
        result
    }
    
    /// Checks if a path exists 
    pub fn path_exists(&self, path: &str) -> bool {
        let canonical = self.normalize_path_canonical(path);
        let exists = self.nodes.contains_key(&canonical);
        serial_println!("DEBUG: TempFS::path_exists - Path '{}' exists: {}", canonical, exists);
        exists
    }
    
    /// Safe, linear path-walking directory creator
    /// Creates a directory and all parent directories as needed
    pub fn ensure_path_exists(&mut self, path: &str) -> Result<(), KernelError> {
        serial_println!("DEBUG: TempFS::ensure_path_exists - Starting for path: '{}'", path);
        
        // Handle root directory as a special case
        if path == "/" {
            serial_println!("DEBUG: TempFS::ensure_path_exists - Root path requested, already exists");
            return Ok(());
        }
        
        // Get canonical path
        let canonical = self.normalize_path_canonical(path);
        
        // Check if path already exists
        if self.path_exists(&canonical) {
            serial_println!("DEBUG: TempFS::ensure_path_exists - Path '{}' already exists", canonical);
            return Ok(());
        }
        
        // Split path into components
        let components: Vec<&str> = canonical.split('/')
            .filter(|s| !s.is_empty())
            .collect();
            
        serial_println!("DEBUG: TempFS::ensure_path_exists - Path components: {:?}", components);
        
        // Build path one component at a time, creating directories as needed
        let mut current_path = String::from("/".to_owned());
        
        for component in components {
            // Build the next path segment
            if current_path == "/" {
                current_path = format!("/{}", component);
            } else {
                current_path = format!("{}/{}", current_path, component);
            }
            
            serial_println!("DEBUG: TempFS::ensure_path_exists - Processing component: '{}', current path: '{}'", component, current_path);
            
            // Check if this path segment exists
            if !self.path_exists(&current_path) {
                serial_println!("DEBUG: TempFS::ensure_path_exists - Creating directory: '{}'", current_path);
                
                // Find the parent path and directory name
                let parent_path = if current_path.rfind('/').unwrap_or(0) == 0 {
                    "/".to_string()
                } else {
                    let last_slash = current_path.rfind('/').unwrap();
                    current_path[..last_slash].to_string()
                };
                
                let dir_name = match current_path.rfind('/') {
                    Some(pos) => &current_path[pos+1..],
                    None => &current_path // Should never happen with our path construction
                };
                
                serial_println!("DEBUG: TempFS::ensure_path_exists - Parent: '{}', Name: '{}'", parent_path, dir_name);
                
                // Create directory node
                let inode = self.next_inode.fetch_add(1, Ordering::SeqCst);
                
                serial_println!("DEBUG: TempFS::ensure_path_exists - Creating directory node with inode: {}", inode);
                let dir_node = TempFsNode {
                    inode,
                    metadata: Metadata::new_directory(),
                    data: NodeData::Directory(BTreeMap::new()),
                };
                
                // Insert node into filesystem
                serial_println!("DEBUG: TempFS::ensure_path_exists - Inserting node for path: '{}'", current_path);
                match self.nodes.insert(current_path.clone(), dir_node) {
                    Some(_) => {
                        serial_println!("DEBUG: TempFS::ensure_path_exists - Node unexpectedly already exists");
                        // Continue anyway, another process might have created it
                    },
                    None => {
                        serial_println!("DEBUG: TempFS::ensure_path_exists - Node inserted successfully");
                        
                        // Update parent directory's entries
                        if let Some(parent_node) = self.nodes.get_mut(&parent_path) {
                            serial_println!("DEBUG: TempFS::ensure_path_exists - Found parent node: '{}'", parent_path);
                            
                            // Safely update parent directory entries
                            match &mut parent_node.data {
                                NodeData::Directory(entries) => {
                                    serial_println!("DEBUG: TempFS::ensure_path_exists - Adding '{}' to parent's directory entries", dir_name);
                                    entries.insert(dir_name.to_string(), inode);
                                },
                                _ => {
                                    serial_println!("ERROR: TempFS::ensure_path_exists - Parent is not a directory!");
                                    return Err(KernelError::NotADirectory);
                                }
                            }
                        } else {
                            // This should never happen with our step-by-step approach
                            serial_println!("ERROR: TempFS::ensure_path_exists - Parent path not found: '{}'", parent_path);
                            return Err(KernelError::GenericError("Parent path not found"));
                        }
                    }
                }
            } else {
                serial_println!("DEBUG: TempFS::ensure_path_exists - Path '{}' already exists, skipping", current_path);
            }
        }
        
        serial_println!("DEBUG: TempFS::ensure_path_exists - Path creation complete for: '{}'", canonical);
        Ok(())
    }

    /// Emergency direct directory creation - bypasses normal path handling
    /// SAFETY: This is only intended for initial filesystem setup
    pub fn direct_create_directory(&mut self, path: &str) -> Result<(), KernelError> {
        serial_println!("DEBUG: TempFS::direct_create_directory - Creating: {}", path);
        
        // Don't try to create root
        if path == "/" {
            return Ok(());
        }

        // First normalize the path string but don't try to handle parents recursively
        let normalized = path.to_string()
            .trim_end_matches('/')
            .to_string();
            
        let normalized = if !normalized.starts_with('/') {
            format!("/{}", normalized)
        } else {
            normalized
        };
        
        // Check if it already exists
        if self.nodes.contains_key(&normalized) {
            serial_println!("DEBUG: TempFS::direct_create_directory - Already exists: {}", normalized);
            return Ok(());
        }
        
        // Split the path to get parent and name components
        let components: Vec<&str> = normalized.split('/')
            .filter(|s| !s.is_empty())
            .collect();
            
        if components.is_empty() {
            return Ok(());
        }
        
        // Create an inode for this directory
        let inode = self.next_inode.fetch_add(1, Ordering::SeqCst);
        
        // Create a new directory node
        let dir_node = TempFsNode {
            inode,
            metadata: Metadata::new_directory(),
            data: NodeData::Directory(BTreeMap::new()),
        };
        
        // Actually add the node to the filesystem
        self.nodes.insert(normalized.clone(), dir_node);
        serial_println!("DEBUG: TempFS::direct_create_directory - Created node: {}", normalized);
        
        // Now try to add to parent, but don't fail if we can't
        let parent_path = components.iter()
            .take(components.len() - 1)
            .fold(String::new(), |mut path, component| {
                path.push('/');
                path.push_str(component);
                path
            });
            
        let parent_path = if parent_path.is_empty() { "/".to_string() } else { parent_path };
        let dir_name = components.last().unwrap().to_string();
        
        serial_println!("DEBUG: TempFS::direct_create_directory - Finding parent: {} for dir: {}", parent_path, dir_name);
        
        // Only try to update parent if it exists
        if let Some(parent_node) = self.nodes.get_mut(&parent_path) {
            if let NodeData::Directory(entries) = &mut parent_node.data {
                entries.insert(dir_name.clone(), inode);
                serial_println!("DEBUG: TempFS::direct_create_directory - Added to parent entries");
            }
        } else {
            serial_println!("DEBUG: TempFS::direct_create_directory - Parent not found, but continuing");
        }
        
        Ok(())
    }
}

impl FileSystem for TempFs {
    fn mount(&mut self) -> Result<(), KernelError> {
        serial_println!("DEBUG: TempFS: Mounting '{}'", self.name);
        Ok(())
    }
    
    fn unmount(&mut self) -> Result<(), KernelError> {
        serial_println!("DEBUG: TempFS: Unmounting '{}'", self.name);
        Ok(())
    }
    
    fn create_file(&mut self, path: &str) -> Result<(), KernelError> {
        let canonical = self.normalize_path_canonical(path);
        serial_println!("DEBUG: TempFS: Creating file '{}'", canonical);
        
        // Check if the file already exists
        if self.nodes.contains_key(&canonical) {
            return Err(KernelError::AlreadyExists);
        }
        
        // Find the parent directory
        let last_slash = canonical.rfind('/').unwrap();
        let parent_path = if last_slash == 0 {
            "/".to_string()
        } else {
            canonical[..last_slash].to_string()
        };
        
        let file_name = &canonical[last_slash+1..];
        
        // Create parent directories if needed
        if !self.nodes.contains_key(&parent_path) {
            self.ensure_path_exists(&parent_path)?;
        }
        
        // Get the parent node
        let parent_node = self.nodes.get_mut(&parent_path)
            .ok_or(KernelError::NotFound)?;
        
        // Ensure the parent is a directory
        match &mut parent_node.data {
            NodeData::Directory(entries) => {
                // Create a new inode
                let inode = self.next_inode.fetch_add(1, Ordering::SeqCst);
                
                // Add to parent directory
                entries.insert(file_name.to_string(), inode);
                
                // Create the file node
                let file_node = TempFsNode {
                    inode,
                    metadata: Metadata::new_file(),
                    data: NodeData::File(Vec::new()),
                };
                
                // Add the file to our nodes
                self.nodes.insert(canonical, file_node);
                
                Ok(())
            },
            _ => Err(KernelError::NotADirectory),
        }
    }
    
    fn create_directory(&mut self, path: &str) -> Result<(), KernelError> {
        serial_println!("DEBUG: TempFS: Creating directory '{}'", path);
        self.ensure_path_exists(path)
    }
    
    fn remove(&mut self, path: &str) -> Result<(), KernelError> {
        let canonical = self.normalize_path_canonical(path);
        
        // Can't remove root
        if canonical == "/" {
            return Err(KernelError::InvalidOperation);
        }
        
        // Check if it exists
        if !self.nodes.contains_key(&canonical) {
            return Err(KernelError::NotFound);
        }
        
        // Get parent path
        let last_slash = canonical.rfind('/').unwrap();
        let parent_path = if last_slash == 0 {
            "/".to_string()
        } else {
            canonical[..last_slash].to_string()
        };
        
        let name = &canonical[last_slash+1..];
        
        // Remove from parent directory
        if let Some(parent) = self.nodes.get_mut(&parent_path) {
            if let NodeData::Directory(entries) = &mut parent.data {
                entries.remove(name);
            }
        }
        
        // Remove the node itself
        self.nodes.remove(&canonical);
        
        Ok(())
    }
    
    fn open(&self, path: &str, write: bool) -> Result<FileHandle, KernelError> {
        let canonical = self.normalize_path_canonical(path);
        
        // Check if it exists
        if !self.nodes.contains_key(&canonical) {
            return Err(KernelError::NotFound);
        }
        
        // Check if it's a file
        let node = self.nodes.get(&canonical).unwrap();
        match &node.data {
            NodeData::File(_) => {
                // Create file handle
                let flags = if write { 
                    crate::fs::vfs::file_flags::WRITE 
                } else { 
                    crate::fs::vfs::file_flags::READ 
                };
                
                // Safe to unwrap because we just checked that the node exists
                Ok(FileHandle::new(&canonical, Arc::new(Mutex::new(self.clone())), flags))
            },
            _ => Err(KernelError::NotAFile),
        }
    }
    
    fn metadata(&self, path: &str) -> Result<Metadata, KernelError> {
        let canonical = self.normalize_path_canonical(path);
        
        // Find the node
        let node = self.nodes.get(&canonical)
            .ok_or(KernelError::NotFound)?;
        
        Ok(node.metadata.clone())
    }
    
    fn read_dir(&self, path: &str) -> Result<Vec<DirEntry>, KernelError> {
        let canonical = self.normalize_path_canonical(path);
        
        // Find the node
        let node = self.nodes.get(&canonical)
            .ok_or(KernelError::NotFound)?;
        
        match &node.data {
            NodeData::Directory(entries) => {
                let mut result = Vec::new();
                
                for (name, &inode) in entries {
                    // Find the child node
                    for (npath, node) in &self.nodes {
                        if node.inode == inode {
                            let node_type = match &node.data {
                                NodeData::File(_) => NodeType::File,
                                NodeData::Directory(_) => NodeType::Directory,
                            };
                            
                            result.push(DirEntry::new(name, node_type, inode));
                            break;
                        }
                    }
                }
                
                Ok(result)
            },
            _ => Err(KernelError::NotADirectory),
        }
    }
    
    fn rename(&mut self, from: &str, to: &str) -> Result<(), KernelError> {
        // Not implemented yet
        Err(KernelError::NotImplemented)
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn total_space(&self) -> u64 {
        1024 * 1024 * 10 // 10 MB
    }
    
    fn available_space(&self) -> u64 {
        1024 * 1024 * 10 // Pretend we have all space available
    }
    
    fn read_at(&self, path: &str, offset: u64, buffer: &mut [u8]) -> Result<usize, KernelError> {
        let canonical = self.normalize_path_canonical(path);
        
        // Find the node
        let node = self.nodes.get(&canonical)
            .ok_or(KernelError::NotFound)?;
        
        match &node.data {
            NodeData::File(data) => {
                let offset = offset as usize;
                
                // Check if we're at EOF
                if offset >= data.len() {
                    return Ok(0);
                }
                
                // Calculate how many bytes we can read
                let bytes_to_read = core::cmp::min(buffer.len(), data.len() - offset);
                
                // Copy the data
                buffer[..bytes_to_read].copy_from_slice(&data[offset..offset + bytes_to_read]);
                
                Ok(bytes_to_read)
            },
            _ => Err(KernelError::NotAFile),
        }
    }
    
    fn write_at(&mut self, path: &str, offset: u64, buffer: &[u8]) -> Result<usize, KernelError> {
        let canonical = self.normalize_path_canonical(path);
        
        // Find the node
        let node = self.nodes.get_mut(&canonical)
            .ok_or(KernelError::NotFound)?;
        
        match &mut node.data {
            NodeData::File(data) => {
                let offset = offset as usize;
                
                // Ensure the file is big enough
                if offset > data.len() {
                    // Pad with zeros
                    data.resize(offset, 0);
                }
                
                // Write the data
                for (i, &byte) in buffer.iter().enumerate() {
                    if offset + i < data.len() {
                        data[offset + i] = byte;
                    } else {
                        data.push(byte);
                    }
                }
                
                // Update metadata
                node.metadata.size = data.len() as u64;
                
                Ok(buffer.len())
            },
            _ => Err(KernelError::NotAFile),
        }
    }
    
    fn is_tempfs(&self) -> bool {
        true
    }
}

// Conversion function needed by fs/mod.rs
pub fn as_tempfs(fs: &mut dyn FileSystem) -> Option<&mut TempFs> {
    // Safe downcast if it's a TempFs
    if fs.is_tempfs() {
        // Safety: We've verified this is a TempFs with is_tempfs
        unsafe { Some(&mut *(fs as *mut dyn FileSystem as *mut TempFs)) }
    } else {
        None
    }
}

// Add basic Clone implementation for TempFs
impl Clone for TempFs {
    fn clone(&self) -> Self {
        // This is a simplistic clone that would create a new empty filesystem
        // In a real implementation, we'd clone all the nodes too
        Self::new(&self.name)
    }
}