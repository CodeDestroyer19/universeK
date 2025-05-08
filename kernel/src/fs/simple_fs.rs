// kernel/src/fs/simple_fs.rs
//! A very simple file system implementation for our OS.
//! This is purposely kept minimal - a real file system would be more complex.

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::format;
use core::fmt;
use super::block_device::BlockDevice;
use spin::Mutex;

/// Represents a file entry in our simple file system
#[derive(Debug, Clone)]
pub struct FileEntry {
    name: String,
    size: usize,
    content: Vec<u8>,
    is_directory: bool,
    // A real file system would have timestamps, permissions, etc.
}

impl FileEntry {
    pub fn new_file(name: &str) -> Self {
        FileEntry {
            name: name.to_string(),
            size: 0,
            content: Vec::new(),
            is_directory: false,
        }
    }
    
    pub fn new_directory(name: &str) -> Self {
        FileEntry {
            name: name.to_string(),
            size: 0,
            content: Vec::new(),
            is_directory: true,
        }
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }
    
    pub fn size(&self) -> usize {
        self.size
    }
    
    pub fn is_directory(&self) -> bool {
        self.is_directory
    }
    
    pub fn content(&self) -> &[u8] {
        &self.content
    }
    
    pub fn write(&mut self, data: &[u8]) {
        self.content = data.to_vec();
        self.size = data.len();
    }
    
    pub fn append(&mut self, data: &[u8]) {
        self.content.extend_from_slice(data);
        self.size = self.content.len();
    }
}

/// Our simple file system structure
#[allow(dead_code)]
pub struct SimpleFileSystem {
    device: Arc<Mutex<dyn BlockDevice>>,
    // In-memory file table (not persistent yet)
    files: BTreeMap<String, FileEntry>,
}

impl SimpleFileSystem {
    /// Create a new file system on the given block device
    pub fn new(device: Arc<Mutex<dyn BlockDevice>>) -> Result<Self, &'static str> {
        let mut fs = SimpleFileSystem {
            device,
            files: BTreeMap::new(),
        };
        
        // Initialize the root directory
        fs.files.insert("/".to_string(), FileEntry::new_directory("/"));
        
        // Create some basic directories for testing
        fs.mkdir("/home")?;
        fs.mkdir("/bin")?;
        fs.mkdir("/etc")?;
        
        // Create a test file
        fs.write_file("/etc/version", b"BearOS v0.1")?;
        
        Ok(fs)
    }
    
    /// Create a new directory
    pub fn mkdir(&mut self, path: &str) -> Result<(), &'static str> {
        if self.files.contains_key(path) {
            return Err("Directory already exists");
        }
        
        // Check if parent directory exists
        let parent_path = parent_dir(path);
        if !parent_path.is_empty() && !self.files.contains_key(parent_path) {
            return Err("Parent directory does not exist");
        }
        
        self.files.insert(path.to_string(), FileEntry::new_directory(path));
        Ok(())
    }
    
    /// Create or overwrite a file with the given content
    pub fn write_file(&mut self, path: &str, content: &[u8]) -> Result<(), &'static str> {
        // Check if parent directory exists
        let parent_path = parent_dir(path);
        if !parent_path.is_empty() && !self.files.contains_key(parent_path) {
            return Err("Parent directory does not exist");
        }
        
        let mut file = FileEntry::new_file(path);
        file.write(content);
        self.files.insert(path.to_string(), file);
        Ok(())
    }
    
    /// Read the content of a file
    pub fn read_file(&self, path: &str) -> Result<&[u8], &'static str> {
        match self.files.get(path) {
            Some(file) if !file.is_directory => Ok(file.content()),
            Some(_) => Err("Path is a directory"),
            None => Err("File not found"),
        }
    }
    
    /// List directory contents
    pub fn list_directory(&self, path: &str) -> Result<Vec<String>, &'static str> {
        // Check if directory exists
        match self.files.get(path) {
            Some(entry) if entry.is_directory => {
                let mut entries = Vec::new();
                let path_with_slash = if path.ends_with('/') { 
                    path.to_string() 
                } else { 
                    format!("{}/", path) 
                };
                
                // For a more efficient implementation, you'd organize directories differently
                for (file_path, _) in self.files.iter() {
                    if file_path != path && 
                       file_path.starts_with(&path_with_slash) && 
                       !file_path[path_with_slash.len()..].contains('/') {
                        // Direct child of this directory
                        entries.push(file_path.clone());
                    }
                }
                
                Ok(entries)
            },
            Some(_) => Err("Path is not a directory"),
            None => Err("Directory not found"),
        }
    }
    
    /// Delete a file or empty directory
    pub fn delete(&mut self, path: &str) -> Result<(), &'static str> {
        if path == "/" {
            return Err("Cannot delete root directory");
        }
        
        match self.files.get(path) {
            Some(entry) if entry.is_directory => {
                // Check if directory is empty
                let path_with_slash = if path.ends_with('/') { 
                    path.to_string() 
                } else { 
                    format!("{}/", path) 
                };
                
                for file_path in self.files.keys() {
                    if file_path != path && file_path.starts_with(&path_with_slash) {
                        return Err("Directory not empty");
                    }
                }
                
                // Directory is empty, remove it
                self.files.remove(path);
                Ok(())
            },
            Some(_) => {
                // Remove the file
                self.files.remove(path);
                Ok(())
            },
            None => Err("File or directory not found"),
        }
    }
    
    /// Persist the file system to the block device (not fully implemented yet)
    pub fn sync(&self) -> Result<(), &'static str> {
        // In a real implementation, this would serialize the file table and contents
        // to the block device. For now, this is a placeholder.
        
        // Our simple implementation keeps everything in memory for simplicity
        Ok(())
    }
}

impl fmt::Debug for SimpleFileSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SimpleFileSystem {{ files: {:?} }}", self.files)
    }
}

// Helper function to get the parent directory of a path
fn parent_dir(path: &str) -> &str {
    match path.rfind('/') {
        Some(pos) if pos > 0 => &path[..pos],
        _ => "", // Root directory or invalid path
    }
} 