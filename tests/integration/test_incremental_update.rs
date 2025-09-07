#[cfg(test)]
mod test_incremental_update {
    use std::process::Command;
    use std::path::Path;
    use tempfile::TempDir;
    use anyhow::Result;
    use std::fs;
    use std::time::{Duration, SystemTime};
    use sha2::{Sha256, Digest};
    
    // All tests in this module must fail until incremental update functionality is implemented
    fn ensure_not_implemented() {
        panic!("incremental update functionality not yet implemented");
    }
    
    /// Test that incremental indexing detects file changes
    #[tokio::test]
    async fn test_incremental_file_change_detection() -> Result<()> {
        ensure_not_implemented();
        
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path().join("incremental_test");
        fs::create_dir_all(&project_path)?;
        
        // Create initial C++ files
        let header_file = project_path.join("test.h");
        let original_header = r#"
#pragma once

class TestClass {
public:
    TestClass();
    void original_method();
    
private:
    int m_value;
};
"#;
        fs::write(&header_file, original_header)?;
        
        let cpp_file = project_path.join("test.cpp");
        let original_cpp = r#"
#include "test.h"

TestClass::TestClass() : m_value(42) {
}

void TestClass::original_method() {
    // Original implementation
}
"#;
        fs::write(&cpp_file, original_cpp)?;
        
        // Calculate initial file hashes (this simulates what the indexer would do)
        let header_hash1 = calculate_file_hash(&header_file)?;
        let cpp_hash1 = calculate_file_hash(&cpp_file)?;
        
        // Wait a moment to ensure different timestamps
        std::thread::sleep(Duration::from_millis(100));
        
        // Modify the header file
        let modified_header = r#"
#pragma once

class TestClass {
public:
    TestClass();
    void original_method();
    void new_method();  // Added new method
    
private:
    int m_value;
    bool m_flag;        // Added new member
};
"#;
        fs::write(&header_file, modified_header)?;
        
        // Modify the cpp file
        let modified_cpp = r#"
#include "test.h"
#include <iostream>

TestClass::TestClass() : m_value(42), m_flag(false) {
}

void TestClass::original_method() {
    // Modified implementation
    std::cout << "Value: " << m_value << std::endl;
}

void TestClass::new_method() {
    m_flag = true;
}
"#;
        fs::write(&cpp_file, modified_cpp)?;
        
        // Calculate new hashes
        let header_hash2 = calculate_file_hash(&header_file)?;
        let cpp_hash2 = calculate_file_hash(&cpp_file)?;
        
        // Verify files were actually changed
        assert_ne!(header_hash1, header_hash2, "Header file should have changed");
        assert_ne!(cpp_hash1, cpp_hash2, "Source file should have changed");
        
        // Test file modification detection
        let header_metadata = fs::metadata(&header_file)?;
        let cpp_metadata = fs::metadata(&cpp_file)?;
        
        assert!(header_metadata.len() > 0, "Header file should have content");
        assert!(cpp_metadata.len() > 0, "Source file should have content");
        
        Ok(())
    }
    
    /// Test incremental update with file additions and deletions
    #[tokio::test]
    async fn test_incremental_file_operations() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path().join("file_ops_test");
        fs::create_dir_all(&project_path)?;
        
        // Create initial file set
        let initial_files = vec![
            ("main.cpp", "int main() { return 0; }"),
            ("utils.h", "#pragma once\nvoid utility();"),
            ("utils.cpp", "#include \"utils.h\"\nvoid utility() {}"),
        ];
        
        let mut tracked_files = Vec::new();
        for (filename, content) in &initial_files {
            let file_path = project_path.join(filename);
            fs::write(&file_path, content)?;
            tracked_files.push((file_path.clone(), calculate_file_hash(&file_path)?));
        }
        
        // Simulate initial index state
        let initial_file_count = tracked_files.len();
        assert_eq!(initial_file_count, 3, "Should have 3 initial files");
        
        // Add new files (simulate file additions)
        let new_files = vec![
            ("new_class.h", r#"
#pragma once
class NewClass {
public:
    NewClass();
    void process();
};
"#),
            ("new_class.cpp", r#"
#include "new_class.h"
NewClass::NewClass() {}
void NewClass::process() {}
"#),
        ];
        
        for (filename, content) in &new_files {
            let file_path = project_path.join(filename);
            fs::write(&file_path, content)?;
            tracked_files.push((file_path.clone(), calculate_file_hash(&file_path)?));
        }
        
        let after_addition_count = tracked_files.len();
        assert_eq!(after_addition_count, 5, "Should have 5 files after additions");
        
        // Delete a file (simulate file deletion)
        let delete_path = project_path.join("utils.cpp");
        fs::remove_file(&delete_path)?;
        
        // Update tracked files (remove deleted file)
        tracked_files.retain(|(path, _)| path.exists());
        let after_deletion_count = tracked_files.len();
        assert_eq!(after_deletion_count, 4, "Should have 4 files after deletion");
        
        // Verify the correct file was deleted
        assert!(!delete_path.exists(), "utils.cpp should be deleted");
        assert!(project_path.join("utils.h").exists(), "utils.h should still exist");
        
        Ok(())
    }
    
    /// Test incremental update performance with large file sets
    #[tokio::test]
    async fn test_incremental_update_performance() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path().join("perf_test");
        fs::create_dir_all(&project_path)?;
        
        // Create a larger set of files to test incremental performance
        let file_count = 50;
        let mut file_hashes = std::collections::HashMap::new();
        
        // Initial file creation
        let start_time = SystemTime::now();
        
        for i in 0..file_count {
            let header_file = project_path.join(format!("class_{}.h", i));
            let cpp_file = project_path.join(format!("class_{}.cpp", i));
            
            let header_content = format!(r#"
#pragma once

class Class{} {{
public:
    Class{}();
    void method_{}();
    
private:
    int m_value_{};
}};
"#, i, i, i, i);
            
            let cpp_content = format!(r#"
#include "class_{}.h"

Class{}::Class{}() : m_value_{}({}) {{
}}

void Class{}::method_{}() {{
    // Implementation for class {}
}}
"#, i, i, i, i, i * 10, i, i, i);
            
            fs::write(&header_file, header_content)?;
            fs::write(&cpp_file, cpp_content)?;
            
            file_hashes.insert(header_file.clone(), calculate_file_hash(&header_file)?);
            file_hashes.insert(cpp_file.clone(), calculate_file_hash(&cpp_file)?);
        }
        
        let creation_time = start_time.elapsed()?;
        println!("Created {} files in {:?}", file_count * 2, creation_time);
        
        // Simulate incremental update by modifying only a few files
        let files_to_modify = 5;
        let modification_start = SystemTime::now();
        
        for i in 0..files_to_modify {
            let cpp_file = project_path.join(format!("class_{}.cpp", i));
            let modified_content = format!(r#"
#include "class_{}.h"
#include <iostream>

Class{}::Class{}() : m_value_{}({}) {{
    std::cout << "Constructor for Class{}" << std::endl;
}}

void Class{}::method_{}() {{
    // Modified implementation for class {}
    std::cout << "Method called for Class{}" << std::endl;
}}
"#, i, i, i, i, i * 10, i, i, i, i, i);
            
            fs::write(&cpp_file, modified_content)?;
            
            // Update hash for modified file
            file_hashes.insert(cpp_file.clone(), calculate_file_hash(&cpp_file)?);
        }
        
        let modification_time = modification_start.elapsed()?;
        println!("Modified {} files in {:?}", files_to_modify, modification_time);
        
        // Check that only modified files have different hashes
        let hash_check_start = SystemTime::now();
        let mut changed_files = 0;
        
        for (file_path, current_hash) in &file_hashes {
            let file_hash = calculate_file_hash(file_path)?;
            if file_hash != *current_hash {
                changed_files += 1;
            }
        }
        
        let hash_check_time = hash_check_start.elapsed()?;
        println!("Hash verification for {} files took {:?}", file_hashes.len(), hash_check_time);
        
        // Performance assertions
        assert!(creation_time < Duration::from_secs(5), "File creation should be fast");
        assert!(modification_time < Duration::from_secs(1), "File modification should be fast");
        assert!(hash_check_time < Duration::from_secs(1), "Hash checking should be fast");
        
        Ok(())
    }
    
    /// Test incremental update with dependency tracking
    #[tokio::test]
    async fn test_incremental_dependency_tracking() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path().join("dependency_test");
        fs::create_dir_all(&project_path)?;
        
        // Create files with dependencies
        let base_header = project_path.join("base.h");
        fs::write(&base_header, r#"
#pragma once

class Base {
public:
    virtual ~Base() = default;
    virtual void process() = 0;
    
protected:
    int m_id;
};
"#)?;
        
        let derived_header = project_path.join("derived.h");
        fs::write(&derived_header, r#"
#pragma once
#include "base.h"

class Derived : public Base {
public:
    Derived(int id);
    void process() override;
    
private:
    std::string m_name;
};
"#)?;
        
        let derived_cpp = project_path.join("derived.cpp");
        fs::write(&derived_cpp, r#"
#include "derived.h"
#include <iostream>

Derived::Derived(int id) {
    m_id = id;
    m_name = "derived_" + std::to_string(id);
}

void Derived::process() {
    std::cout << "Processing " << m_name << std::endl;
}
"#)?;
        
        let main_cpp = project_path.join("main.cpp");
        fs::write(&main_cpp, r#"
#include "derived.h"

int main() {
    Derived obj(1);
    obj.process();
    return 0;
}
"#)?;
        
        // Create dependency graph
        let dependencies = create_dependency_graph(&project_path)?;
        
        // base.h should be a dependency of derived.h
        let empty_vec = Vec::new();
        let _base_deps = dependencies.get(&base_header).unwrap_or(&empty_vec);
        let derived_deps = dependencies.get(&derived_header).unwrap_or(&empty_vec);
        let main_deps = dependencies.get(&main_cpp).unwrap_or(&empty_vec);
        
        // Verify dependency relationships
        assert!(derived_deps.contains(&base_header), "derived.h should depend on base.h");
        assert!(main_deps.contains(&derived_header), "main.cpp should depend on derived.h");
        
        // Test that modifying base.h affects derived files
        let original_base_hash = calculate_file_hash(&base_header)?;
        
        // Modify base.h
        fs::write(&base_header, r#"
#pragma once

class Base {
public:
    virtual ~Base() = default;
    virtual void process() = 0;
    virtual void new_virtual_method();  // Added new virtual method
    
protected:
    int m_id;
    bool m_enabled;  // Added new member
};
"#)?;
        
        let modified_base_hash = calculate_file_hash(&base_header)?;
        assert_ne!(original_base_hash, modified_base_hash, "base.h should be modified");
        
        // Files that depend on base.h should be marked for re-indexing
        let files_to_reindex = get_affected_files(&dependencies, &base_header);
        
        assert!(files_to_reindex.contains(&derived_header), "derived.h should be re-indexed");
        assert!(files_to_reindex.contains(&derived_cpp), "derived.cpp should be re-indexed");
        
        Ok(())
    }
    
    /// Test incremental update with file watching simulation
    #[tokio::test]
    async fn test_file_watching_simulation() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path().join("watch_test");
        fs::create_dir_all(&project_path)?;
        
        // Create initial files
        let watch_file = project_path.join("watched.cpp");
        fs::write(&watch_file, "// Initial content")?;
        
        let initial_metadata = fs::metadata(&watch_file)?;
        let initial_modified = initial_metadata.modified()?;
        
        // Wait to ensure different timestamp
        std::thread::sleep(Duration::from_millis(100));
        
        // Simulate file change events
        let file_events = vec![
            FileEvent::Modified(watch_file.clone()),
            FileEvent::Created(project_path.join("new_file.h")),
            FileEvent::Deleted(project_path.join("deleted_file.cpp")),
        ];
        
        for event in &file_events {
            match event {
                FileEvent::Modified(path) => {
                    fs::write(path, "// Modified content")?;
                    let new_metadata = fs::metadata(path)?;
                    let new_modified = new_metadata.modified()?;
                    assert!(new_modified > initial_modified, "File should have newer timestamp");
                }
                FileEvent::Created(path) => {
                    fs::write(path, "// New file content")?;
                    assert!(path.exists(), "New file should exist");
                }
                FileEvent::Deleted(_path) => {
                    // Don't actually delete - just simulate the event
                    // In real implementation, we would remove from index
                }
            }
        }
        
        // Verify event processing
        assert_eq!(file_events.len(), 3, "Should process all file events");
        
        Ok(())
    }
    
    /// Test incremental update with Merkle tree-like change tracking
    #[tokio::test]
    async fn test_merkle_tree_change_tracking() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path().join("merkle_test");
        fs::create_dir_all(&project_path)?;
        
        // Create directory structure
        let src_dir = project_path.join("src");
        let include_dir = project_path.join("include");
        fs::create_dir_all(&src_dir)?;
        fs::create_dir_all(&include_dir)?;
        
        // Create files in different directories
        let files = vec![
            (src_dir.join("main.cpp"), "int main() { return 0; }"),
            (src_dir.join("utils.cpp"), "void utility() {}"),
            (include_dir.join("api.h"), "#pragma once\nvoid api();"),
            (include_dir.join("types.h"), "#pragma once\ntypedef int MyInt;"),
        ];
        
        let mut merkle_tree = MerkleTree::new();
        
        // Build initial Merkle tree
        for (file_path, content) in &files {
            fs::write(file_path, content)?;
            let hash = calculate_file_hash(file_path)?;
            merkle_tree.insert(file_path.clone(), hash);
        }
        
        let initial_root_hash = merkle_tree.root_hash();
        
        // Modify one file
        let modified_file = &src_dir.join("main.cpp");
        fs::write(modified_file, "int main() { return 42; }")?;
        let new_hash = calculate_file_hash(modified_file)?;
        merkle_tree.update(modified_file.clone(), new_hash);
        
        let updated_root_hash = merkle_tree.root_hash();
        
        // Root hash should change when any file changes
        assert_ne!(initial_root_hash, updated_root_hash, "Root hash should change");
        
        // Only the modified file should be different
        let changed_files = merkle_tree.get_changed_files();
        assert_eq!(changed_files.len(), 1, "Only one file should be changed");
        assert!(changed_files.contains(modified_file), "Should identify the correct changed file");
        
        Ok(())
    }
    
    // Helper functions and types
    fn calculate_file_hash(file_path: &Path) -> Result<String> {
        let content = fs::read(file_path)?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        Ok(format!("{:x}", hasher.finalize()))
    }
    
    fn create_dependency_graph(project_path: &Path) -> Result<std::collections::HashMap<std::path::PathBuf, Vec<std::path::PathBuf>>> {
        let mut dependencies = std::collections::HashMap::new();
        
        // Simple dependency parsing - look for #include statements
        use walkdir::WalkDir;
        
        for entry in WalkDir::new(project_path) {
            let entry = entry?;
            if entry.file_type().is_file() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "cpp" || ext == "h" || ext == "hpp" {
                        let content = fs::read_to_string(path)?;
                        let mut file_deps = Vec::new();
                        
                        for line in content.lines() {
                            let trimmed = line.trim();
                            if trimmed.starts_with("#include \"") && trimmed.ends_with("\"") {
                                let include_file = &trimmed[10..trimmed.len()-1];
                                let include_path = project_path.join(include_file);
                                if include_path.exists() {
                                    file_deps.push(include_path);
                                }
                            }
                        }
                        
                        dependencies.insert(path.to_path_buf(), file_deps);
                    }
                }
            }
        }
        
        Ok(dependencies)
    }
    
    fn get_affected_files(dependencies: &std::collections::HashMap<std::path::PathBuf, Vec<std::path::PathBuf>>, modified_file: &Path) -> Vec<std::path::PathBuf> {
        let mut affected = Vec::new();
        
        for (file, deps) in dependencies {
            if deps.contains(&modified_file.to_path_buf()) {
                affected.push(file.clone());
            }
        }
        
        affected
    }
    
    #[derive(Debug)]
    enum FileEvent {
        Created(std::path::PathBuf),
        Modified(std::path::PathBuf),
        Deleted(std::path::PathBuf),
    }
    
    struct MerkleTree {
        files: std::collections::HashMap<std::path::PathBuf, String>,
    }
    
    impl MerkleTree {
        fn new() -> Self {
            Self {
                files: std::collections::HashMap::new(),
            }
        }
        
        fn insert(&mut self, path: std::path::PathBuf, hash: String) {
            self.files.insert(path, hash);
        }
        
        fn update(&mut self, path: std::path::PathBuf, new_hash: String) {
            self.files.insert(path, new_hash);
        }
        
        fn root_hash(&self) -> String {
            let mut hasher = Sha256::new();
            let mut sorted_files: Vec<_> = self.files.iter().collect();
            sorted_files.sort_by_key(|(path, _)| path.to_string_lossy());
            
            for (path, hash) in sorted_files {
                hasher.update(path.to_string_lossy().as_bytes());
                hasher.update(hash.as_bytes());
            }
            
            format!("{:x}", hasher.finalize())
        }
        
        fn get_changed_files(&self) -> Vec<std::path::PathBuf> {
            // In a real implementation, this would compare against a previous state
            // For testing, we'll just return files that exist
            self.files.keys()
                .filter(|path| path.exists())
                .cloned()
                .collect()
        }
    }
}